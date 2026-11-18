use std::env;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use smoooth::storage::FileLock;

#[tokio::test]
async fn test_subprocess_death_lock_recovery() {
    let temp_dir = std::env::temp_dir();
    let lock_path = temp_dir.join(format!(
        "smoooth_subprocess_lock_{}.lock",
        uuid::Uuid::new_v4().simple()
    ));

    let current_exe = env::current_exe().expect("failed to get current_exe");

    // Spawn Subprocess 1 in lock-holder mode
    let mut child1 = Command::new(&current_exe)
        .arg("--exact")
        .arg("helper_lock_worker")
        .arg("--nocapture")
        .env("SMOOOTH_LOCK_TEST_PATH", &lock_path)
        .env("SMOOOTH_LOCK_TEST_ROLE", "hold_until_killed")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn child1");

    let stdout = child1.stdout.take().expect("failed to take child1 stdout");
    let reader = BufReader::new(stdout);
    let mut lock_held = false;
    for line in reader.lines() {
        if let Ok(l) = line {
            if l.contains("LOCK_HELD") {
                lock_held = true;
                break;
            }
        }
    }
    assert!(lock_held, "Child 1 must report LOCK_HELD");

    // Verify parent cannot acquire lock while Child 1 holds it
    let attempt = FileLock::acquire(&lock_path, Duration::from_millis(100)).await;
    assert!(
        attempt.is_err(),
        "Parent must NOT acquire lock while child 1 is holding it"
    );

    // Forcibly kill Child 1 (SIGKILL on Unix / TerminateProcess on Windows)
    // Child 1 is killed abruptly by kernel; no Rust Drop or destructor runs.
    child1.kill().expect("failed to kill child1");
    let _ = child1.wait().expect("failed to wait on killed child1");

    // Verify lock file STILL exists on disk (was not deleted or truncated)
    assert!(
        lock_path.exists(),
        "Lock file must persist on disk after process death"
    );

    // Spawn Subprocess 2 to verify a completely separate process acquires the lock
    let mut child2 = Command::new(&current_exe)
        .arg("--exact")
        .arg("helper_lock_worker")
        .arg("--nocapture")
        .env("SMOOOTH_LOCK_TEST_PATH", &lock_path)
        .env("SMOOOTH_LOCK_TEST_ROLE", "acquire_and_exit")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn child2");

    let child2_stdout = child2.stdout.take().expect("failed to take child2 stdout");
    let reader2 = BufReader::new(child2_stdout);
    let mut acquire_success = false;
    for line in reader2.lines() {
        if let Ok(l) = line {
            if l.contains("ACQUIRE_SUCCESS") {
                acquire_success = true;
                break;
            }
        }
    }
    assert!(
        acquire_success,
        "Child 2 must report ACQUIRE_SUCCESS after Child 1 death"
    );

    let status2 = child2.wait().expect("failed to wait on child2");
    assert!(status2.success(), "Child 2 must exit successfully");

    // Finally, parent acquires the lock directly to confirm full recovery
    let parent_lock = FileLock::acquire(&lock_path, Duration::from_secs(3))
        .await
        .expect("Parent must acquire lock after Child 2 exits");
    drop(parent_lock);

    let _ = std::fs::remove_file(&lock_path);
}

#[tokio::test]
async fn helper_lock_worker() {
    let Ok(lock_path_str) = env::var("SMOOOTH_LOCK_TEST_PATH") else {
        return; // Normal test execution skips this helper
    };
    let Ok(role) = env::var("SMOOOTH_LOCK_TEST_ROLE") else {
        return;
    };
    let lock_path = PathBuf::from(lock_path_str);

    match role.as_str() {
        "hold_until_killed" => {
            let _lock = FileLock::acquire(&lock_path, Duration::from_secs(10))
                .await
                .expect("helper hold_until_killed failed to acquire lock");
            let _ = writeln!(std::io::stdout(), "LOCK_HELD");
            let _ = std::io::stdout().flush();
            // Block indefinitely until forcibly terminated by OS signal / TerminateProcess
            loop {
                std::thread::sleep(Duration::from_millis(500));
            }
        }
        "acquire_and_exit" => {
            let _lock = FileLock::acquire(&lock_path, Duration::from_secs(5))
                .await
                .expect("helper acquire_and_exit failed to acquire lock");
            let _ = writeln!(std::io::stdout(), "ACQUIRE_SUCCESS");
            let _ = std::io::stdout().flush();
            std::process::exit(0);
        }
        _ => {}
    }
}
