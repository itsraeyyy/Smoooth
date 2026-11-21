use crate::error::{Result, SmooothError};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};
use tokio_postgres::{Client, NoTls};

/// Bounded, leak-free connection pool for tokio-postgres.
#[derive(Clone)]
pub struct PostgresPool {
    inner: Arc<PoolInner>,
}

struct PoolInner {
    config_str: String,
    schema: Option<String>,
    clients: Mutex<Vec<Client>>,
    semaphore: Arc<Semaphore>,
}

pub struct PooledClient {
    client: Option<Client>,
    pool: Arc<PoolInner>,
    _permit: OwnedSemaphorePermit,
}

impl Deref for PooledClient {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        self.client
            .as_ref()
            .expect("PooledClient client is always present")
    }
}

impl DerefMut for PooledClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.client
            .as_mut()
            .expect("PooledClient client is always present")
    }
}

impl Drop for PooledClient {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            if !client.is_closed() {
                match self.pool.clients.try_lock() {
                    Ok(mut lock) => {
                        lock.push(client);
                    }
                    Err(_) => {
                        let pool = Arc::clone(&self.pool);
                        tokio::spawn(async move {
                            let mut lock = pool.clients.lock().await;
                            lock.push(client);
                        });
                    }
                }
            }
        }
    }
}

impl PostgresPool {
    /// Creates a new PostgresPool with the specified connection string and max connection limit.
    pub fn new(config_str: impl Into<String>, max_connections: usize) -> Self {
        Self::new_with_schema(config_str, max_connections, None)
    }

    /// Creates a new PostgresPool configured with an optional schema search_path.
    pub fn new_with_schema(
        config_str: impl Into<String>,
        max_connections: usize,
        schema: Option<String>,
    ) -> Self {
        let max_conns = max_connections.max(1);
        Self {
            inner: Arc::new(PoolInner {
                config_str: config_str.into(),
                schema,
                clients: Mutex::new(Vec::with_capacity(max_conns)),
                semaphore: Arc::new(Semaphore::new(max_conns)),
            }),
        }
    }

    /// Acquires an exclusive client connection from the pool.
    pub async fn get(&self) -> Result<PooledClient> {
        let permit = self
            .inner
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| {
                SmooothError::Storage(format!("Failed to acquire connection permit: {}", e))
            })?;

        // 1. Check existing idle clients
        {
            let mut clients = self.inner.clients.lock().await;
            while let Some(client) = clients.pop() {
                if !client.is_closed() {
                    return Ok(PooledClient {
                        client: Some(client),
                        pool: Arc::clone(&self.inner),
                        _permit: permit,
                    });
                }
            }
        }

        // 2. Establish new connection if queue empty or drained of closed sockets
        let (client, connection) = tokio_postgres::connect(&self.inner.config_str, NoTls)
            .await
            .map_err(|e| SmooothError::Storage(format!("PostgreSQL connection failure: {}", e)))?;

        tokio::spawn(async move {
            let _ = connection.await;
        });

        // 3. Configure search_path if scoped schema specified
        if let Some(schema) = &self.inner.schema {
            client
                .batch_execute(&format!("SET search_path TO \"{}\", public;", schema))
                .await
                .map_err(|e| {
                    SmooothError::Storage(format!(
                        "Failed to set search_path to '{}': {}",
                        schema, e
                    ))
                })?;
        }

        Ok(PooledClient {
            client: Some(client),
            pool: Arc::clone(&self.inner),
            _permit: permit,
        })
    }

    /// Returns the configured schema if set.
    pub fn schema(&self) -> Option<&str> {
        self.inner.schema.as_deref()
    }
}
