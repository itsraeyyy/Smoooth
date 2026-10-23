use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

use bollard::container::LogOutput;
use bollard::exec::{CreateExecOptions, StartExecResults};
use futures_util::StreamExt;

use smoooth::adapters::docker::client::DockerClient;
use smoooth::adapters::docker::containers::{ContainerManager, ContainerSpec};
use smoooth::adapters::docker::images::ImageManager;
use smoooth::adapters::docker::volumes::VolumeManager;
use smoooth::adapters::docker::DockerAdapter;
use smoooth::adapters::mongo::MongoAdapter;
use smoooth::adapters::postgres::PostgresAdapter;
use smoooth::adapters::redis::RedisAdapter;
use smoooth::adapters::ProviderAdapter;
use smoooth::models::connection::VolumeMount;

async fn get_verified_docker_client() -> Option<DockerClient> {
    match DockerClient::try_new() {
        Ok(c) => match c.ping().await {
            Ok(_) => Some(c),
            Err(e) => {
                if std::env::var("SMOOOTH_REQUIRE_DOCKER").unwrap_or_default() == "1" {
                    panic!("SMOOOTH_REQUIRE_DOCKER=1 is set, but Docker ping failed: {e}");
                }
                None
            }
        },
        Err(e) => {
            if std::env::var("SMOOOTH_REQUIRE_DOCKER").unwrap_or_default() == "1" {
                panic!("SMOOOTH_REQUIRE_DOCKER=1 is set, but DockerClient::try_new() failed: {e}");
            }
            None
        }
    }
}

async fn docker_exec(
    docker: &bollard::Docker,
    container_name: &str,
    cmd: Vec<&str>,
) -> (i64, String) {
    let exec = docker
        .create_exec(
            container_name,
            CreateExecOptions {
                cmd: Some(cmd.into_iter().map(|s| s.to_string()).collect()),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await
        .expect("create_exec should succeed");

    let mut output = String::new();
    if let StartExecResults::Attached { output: mut stream, .. } = docker
        .start_exec(&exec.id, None)
        .await
        .expect("start_exec should succeed")
    {
        while let Some(Ok(chunk)) = stream.next().await {
            match chunk {
                LogOutput::StdOut { message } => {
                    output.push_str(&String::from_utf8_lossy(&message));
                }
                LogOutput::StdErr { message } => {
                    output.push_str(&String::from_utf8_lossy(&message));
                }
                _ => {}
            }
        }
    }

    let inspect = docker
        .inspect_exec(&exec.id)
        .await
        .expect("inspect_exec should succeed");
    let exit_code = inspect.exit_code.unwrap_or(-1);
    (exit_code, output)
}

#[tokio::test]
async fn test_live_postgres_persistence_across_container_replacement() {
    let client = match get_verified_docker_client().await {
        Some(c) => c,
        None => return,
    };

    let docker = Arc::new(DockerAdapter::with_client(client.clone()));
    let postgres = PostgresAdapter::new(docker.clone());

    let service_name = "smoooth_live_pg_pers_svc";
    let vol_name = "smoooth_live_pg_pers_vol";
    let test_port = 15433u16;

    let config = serde_json::json!({
        "database": "pers_test_db",
        "user": "pers_test_user",
        "password": "pers_test_secure_pass_123",
        "port": test_port,
        "volume_name": vol_name,
        "workspace_id": "ws-live-test"
    });

    // Clean up any stale artifacts
    let _ = ContainerManager::stop_and_remove(&client, service_name).await;
    let _ = VolumeManager::remove_volume(&client, vol_name, true).await;

    // 1. Provision initial PostgreSQL instance
    println!(">>> Provisioning initial PostgreSQL container with persistent volume...");
    let res1 = postgres.provision(service_name, &config, true).await;
    assert!(res1.is_ok(), "Initial PG provisioning must succeed: {:?}", res1.as_ref().err());
    let res_obj = res1.unwrap();
    assert!(res_obj.is_persistent);

    // Verify health
    let h1 = postgres.health_check(&res_obj.id).await.unwrap();
    assert!(h1.is_healthy, "PG initial health check must be healthy");

    // 2. Connect via tokio-postgres and write a test record
    println!(">>> Connecting to PostgreSQL to write persistent record...");
    let conn_str = format!(
        "host=127.0.0.1 port={} user=pers_test_user password=pers_test_secure_pass_123 dbname=pers_test_db",
        test_port
    );
    let (pg_client, connection) = tokio_postgres::connect(&conn_str, tokio_postgres::NoTls)
        .await
        .expect("Failed to connect to PG");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PG connection error: {}", e);
        }
    });

    pg_client
        .batch_execute("CREATE TABLE live_check (id INT PRIMARY KEY, val TEXT); INSERT INTO live_check VALUES (1, 'smoooth_persisted_record');")
        .await
        .expect("Writing initial test record to PG failed");

    let row = pg_client
        .query_one("SELECT val FROM live_check WHERE id = 1", &[])
        .await
        .expect("Query failed");
    let initial_val: &str = row.get(0);
    assert_eq!(initial_val, "smoooth_persisted_record");
    drop(pg_client);

    // 3. Stop and remove the container without destroying the volume
    println!(">>> Destroying PG container while preserving persistent volume...");
    let stop_res = ContainerManager::stop_and_remove(&client, service_name).await;
    assert!(stop_res.is_ok(), "Container removal must succeed");

    // 4. Provision replacement container bound to the same persistent volume
    println!(">>> Provisioning replacement PG container on existing volume...");
    let res2 = postgres.provision(service_name, &config, true).await;
    assert!(res2.is_ok(), "Replacement PG provisioning must succeed: {:?}", res2.as_ref().err());

    // Wait for replacement readiness
    let h2 = postgres.health_check(service_name).await.unwrap();
    assert!(h2.is_healthy, "Replacement PG container must be healthy");

    // 5. Connect to replacement PG container and verify data survived intact
    println!(">>> Verifying record persisted in replacement container...");
    let (pg_client2, connection2) = tokio_postgres::connect(&conn_str, tokio_postgres::NoTls)
        .await
        .expect("Failed to connect to replacement PG");
    tokio::spawn(async move {
        if let Err(e) = connection2.await {
            eprintln!("PG replacement connection error: {}", e);
        }
    });

    let row2 = pg_client2
        .query_one("SELECT val FROM live_check WHERE id = 1", &[])
        .await
        .expect("Querying persisted record from replacement PG container failed");
    let recovered_val: &str = row2.get(0);
    assert_eq!(
        recovered_val, "smoooth_persisted_record",
        "Persistent record must survive container replacement"
    );

    // Cleanup
    let _ = ContainerManager::stop_and_remove(&client, service_name).await;
    let _ = VolumeManager::remove_volume(&client, vol_name, true).await;
    println!(">>> PostgreSQL persistence verification completed successfully.");
}

#[tokio::test]
async fn test_live_redis_persistence_and_health_check() {
    let client = match get_verified_docker_client().await {
        Some(c) => c,
        None => return,
    };

    let docker = Arc::new(DockerAdapter::with_client(client.clone()));
    let redis = RedisAdapter::new(docker.clone());

    let service_name = "smoooth_live_redis_pers_svc";
    let vol_name = "smoooth_live_redis_pers_vol";
    let test_port = 16379u16;

    let config = serde_json::json!({
        "port": test_port,
        "volume_name": vol_name,
        "workspace_id": "ws-live-test"
    });

    // Clean up
    let _ = ContainerManager::stop_and_remove(&client, service_name).await;
    let _ = VolumeManager::remove_volume(&client, vol_name, true).await;

    // 1. Provision Redis instance
    println!(">>> Provisioning Redis container with persistent volume...");
    let res = redis.provision(service_name, &config, true).await;
    assert!(res.is_ok(), "Redis provisioning must succeed");

    // 2. Connect via TCP and execute Redis SET / SAVE commands
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", test_port))
        .await
        .expect("Failed to connect to Redis port");

    // SET smoooth_key redis_persisted_data
    stream
        .write_all(b"*3\r\n$3\r\nSET\r\n$11\r\nsmoooth_key\r\n$20\r\nredis_persisted_data\r\n")
        .await
        .expect("Failed to send SET command");
    let mut buf = [0u8; 128];
    let n = stream.read(&mut buf).await.expect("Failed to read SET response");
    let set_resp = String::from_utf8_lossy(&buf[..n]);
    assert!(set_resp.contains("+OK"), "SET must return +OK");

    // SAVE snapshot
    stream
        .write_all(b"*1\r\n$4\r\nSAVE\r\n")
        .await
        .expect("Failed to send SAVE command");
    let n = stream.read(&mut buf).await.expect("Failed to read SAVE response");
    let save_resp = String::from_utf8_lossy(&buf[..n]);
    assert!(save_resp.contains("+OK"), "SAVE must return +OK");

    // GET smoooth_key
    stream
        .write_all(b"*2\r\n$3\r\nGET\r\n$11\r\nsmoooth_key\r\n")
        .await
        .expect("Failed to send GET command");
    let n = stream.read(&mut buf).await.expect("Failed to read GET response");
    let get_resp = String::from_utf8_lossy(&buf[..n]);
    assert!(
        get_resp.contains("redis_persisted_data"),
        "GET must return persisted value"
    );
    drop(stream);

    // 3. Health check verification
    let health = redis.health_check(service_name).await.unwrap();
    assert!(health.is_healthy, "Redis health check must be healthy");

    // Cleanup
    let _ = ContainerManager::stop_and_remove(&client, service_name).await;
    let _ = VolumeManager::remove_volume(&client, vol_name, true).await;
    println!(">>> Redis persistence verification completed successfully.");
}

#[tokio::test]
async fn test_live_mongodb_provision_query_and_persistence() {
    let client = match get_verified_docker_client().await {
        Some(c) => c,
        None => return,
    };

    let docker = Arc::new(DockerAdapter::with_client(client.clone()));
    let mongo = MongoAdapter::new(docker.clone());

    let service_name = "smoooth-mongo-live-test";
    let vol_name = "smoooth-live-test-data";
    let test_port = 27018u16;

    let config = serde_json::json!({
        "user": "rootuser",
        "password": "rootpassword123",
        "database": "appdb",
        "ports": {
            "27017": test_port
        }
    });

    // Clean up
    let _ = ContainerManager::stop_and_remove(&client, service_name).await;
    let _ = VolumeManager::remove_volume(&client, vol_name, true).await;

    // 1. Provision MongoDB instance
    println!(">>> Provisioning MongoDB container (mongo:7-jammy) with persistent volume...");
    let res1 = mongo.provision("live-test", &config, true).await;
    assert!(res1.is_ok(), "MongoDB provisioning must succeed: {:?}", res1.err());

    // 2. Insert test document using mongosh inside the container
    println!(">>> Inserting test document into MongoDB via mongosh...");
    let mut inserted = false;
    let mut last_out = String::new();
    for _ in 0..60 {
        let (exit1, out1) = docker_exec(
            client.inner(),
            service_name,
            vec![
                "mongosh",
                "-u",
                "rootuser",
                "-p",
                "rootpassword123",
                "--authenticationDatabase",
                "admin",
                "appdb",
                "--eval",
                "db.items.insertOne({ key: 'mongo_marker', val: 'survives_replacement' });",
            ],
        )
        .await;
        if exit1 == 0 && out1.contains("acknowledged: true") {
            inserted = true;
            break;
        }
        last_out = out1;
        sleep(Duration::from_millis(500)).await;
    }
    assert!(inserted, "Inserting document into Mongo must succeed. Last output: {}", last_out);

    // 3. Stop and remove the container without destroying the volume
    println!(">>> Destroying MongoDB container while preserving volume '{}'...", vol_name);
    let stop_res = ContainerManager::stop_and_remove(&client, service_name).await;
    assert!(stop_res.is_ok(), "Mongo container removal must succeed");

    // 4. Provision replacement MongoDB container bound to existing volume
    println!(">>> Provisioning replacement MongoDB container with same volume...");
    let res2 = mongo.provision("live-test", &config, true).await;
    assert!(res2.is_ok(), "Replacement MongoDB provisioning must succeed: {:?}", res2.err());

    // 5. Query document from replacement container to verify persistence
    println!(">>> Verifying data survived in replacement MongoDB container...");
    let mut queried = false;
    let mut last_query_out = String::new();
    for _ in 0..60 {
        let (exit2, out2) = docker_exec(
            client.inner(),
            service_name,
            vec![
                "mongosh",
                "-u",
                "rootuser",
                "-p",
                "rootpassword123",
                "--authenticationDatabase",
                "admin",
                "appdb",
                "--eval",
                "const doc = db.items.findOne({ key: 'mongo_marker' }); print('DOC_VAL:' + doc.val);",
            ],
        )
        .await;
        if exit2 == 0 && out2.contains("DOC_VAL:survives_replacement") {
            queried = true;
            break;
        }
        last_query_out = out2;
        sleep(Duration::from_millis(500)).await;
    }
    assert!(
        queried,
        "MongoDB record must survive container replacement! Output: {}",
        last_query_out
    );

    // Cleanup
    let _ = ContainerManager::stop_and_remove(&client, service_name).await;
    let _ = VolumeManager::remove_volume(&client, vol_name, true).await;
    println!(">>> MongoDB persistence verification completed successfully.");
}

#[tokio::test]
async fn test_live_sqlite_persistent_volume_retention_across_replacement() {
    let client = match get_verified_docker_client().await {
        Some(c) => c,
        None => return,
    };

    let vol_name = "smoooth_live_sqlite_vol";
    let c1_name = "smoooth_sqlite_writer";
    let c2_name = "smoooth_sqlite_reader";

    // Clean up
    let _ = ContainerManager::stop_and_remove(&client, c1_name).await;
    let _ = ContainerManager::stop_and_remove(&client, c2_name).await;
    let _ = VolumeManager::remove_volume(&client, vol_name, true).await;

    // 1. Ensure volume
    VolumeManager::ensure_volume(&client, vol_name, "ws-sqlite", true)
        .await
        .expect("Volume creation must succeed");

    // 2. Launch initial container that installs sqlite3, creates /data/app.db, and writes data
    println!(">>> Starting first container to write to SQLite database file on persistent volume...");
    let spec1 = ContainerSpec {
        name: c1_name.to_string(),
        image: "alpine:latest".to_string(),
        cmd: Some(vec![
            "sh".to_string(),
            "-c".to_string(),
            "apk add --no-cache sqlite && sqlite3 /data/app.db \"CREATE TABLE persistent_kv (k TEXT PRIMARY KEY, v TEXT); INSERT INTO persistent_kv VALUES ('state_key', 'sqlite_persisted_999');\" && touch /data/writer_done.txt && sleep 120".to_string(),
        ]),
        env: vec!["DATABASE_URL=sqlite:///data/app.db".to_string()],
        port_bindings: HashMap::new(),
        custom_port_bindings: Vec::new(),
        volume_mounts: vec![VolumeMount::new(vol_name.to_string(), "/data".to_string(), false)],
        network_name: None,
        labels: HashMap::new(),
        restart_policy: None,
        memory_bytes: None,
        cpu_shares: None,
        nano_cpus: None,
        health_spec: None,
    };

    ImageManager::ensure_image(&client, "alpine:latest", None)
        .await
        .expect("Alpine image pull must succeed");
    ContainerManager::create_and_start(&client, &spec1)
        .await
        .expect("Starting container 1 must succeed");

    // Poll until writer container completes SQLite initialization
    let mut writer_done = false;
    for _ in 0..120 {
        let (code, _) = docker_exec(client.inner(), c1_name, vec!["ls", "/data/writer_done.txt"]).await;
        if code == 0 {
            writer_done = true;
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }
    assert!(writer_done, "Writer container must finish writing to SQLite database");

    // 3. Stop and completely destroy writer container
    println!(">>> Completely destroying writer container...");
    ContainerManager::stop_and_remove(&client, c1_name)
        .await
        .expect("Stopping container 1 must succeed");

    // 4. Launch independent reader container mounting the SAME volume to prove persistence
    println!(">>> Starting NEW replacement reader container mounting same persistent volume...");
    let spec2 = ContainerSpec {
        name: c2_name.to_string(),
        image: "alpine:latest".to_string(),
        cmd: Some(vec![
            "sh".to_string(),
            "-c".to_string(),
            "apk add --no-cache sqlite && sqlite3 /data/app.db \"SELECT v FROM persistent_kv WHERE k='state_key';\" > /data/output.txt && touch /data/reader_done.txt && sleep 120".to_string(),
        ]),
        env: vec!["DATABASE_URL=sqlite:///data/app.db".to_string()],
        port_bindings: HashMap::new(),
        custom_port_bindings: Vec::new(),
        volume_mounts: vec![VolumeMount::new(vol_name.to_string(), "/data".to_string(), false)],
        network_name: None,
        labels: HashMap::new(),
        restart_policy: None,
        memory_bytes: None,
        cpu_shares: None,
        nano_cpus: None,
        health_spec: None,
    };

    ContainerManager::create_and_start(&client, &spec2)
        .await
        .expect("Starting reader container must succeed");

    // Poll until reader container completes query
    let mut reader_done = false;
    for _ in 0..120 {
        let (code, _) = docker_exec(client.inner(), c2_name, vec!["ls", "/data/reader_done.txt"]).await;
        if code == 0 {
            reader_done = true;
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }
    assert!(reader_done, "Reader container must finish reading from SQLite database");

    // 5. Query /data/output.txt inside container 2
    let (exit_code, output) = docker_exec(
        client.inner(),
        c2_name,
        vec!["cat", "/data/output.txt"],
    )
    .await;

    assert_eq!(exit_code, 0, "Reading output.txt must succeed");
    assert!(
        output.contains("sqlite_persisted_999"),
        "SQLite data must persist across container replacement! Got: {}",
        output
    );

    // Cleanup
    let _ = ContainerManager::stop_and_remove(&client, c2_name).await;
    let _ = VolumeManager::remove_volume(&client, vol_name, true).await;
    println!(">>> SQLite persistent volume verification completed successfully.");
}
