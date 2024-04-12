use color_eyre::{eyre::eyre, Report, Result as AnyResult};
use redis::{Client, Commands, Connection, ConnectionInfo, RedisConnectionInfo};

use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};

pub struct RedisClient {
    connection: Connection,
}

impl RedisClient {
    pub fn new(
        host: String,
        port: u16,
        db: i64,
        username: Option<String>,
        password: Option<String>,
    ) -> AnyResult<Self> {
        let connection = ConnectionInfo {
            addr: redis::ConnectionAddr::Tcp(host, port),
            redis: RedisConnectionInfo {
                db,
                username,
                password,
            },
        };
        Ok(RedisClient {
            connection: Client::open(connection)?.get_connection()?,
        })
    }

    pub fn get(&mut self, key: &str) -> AnyResult<Option<String>> {
        self.connection.get(key).map_err(|e| eyre!(e))
    }

    pub fn set(&mut self, key: &str, value: &str) -> AnyResult<()> {
        self.connection.set(key, value).map_err(|e| eyre!(e))
    }
}

pub enum RedisCommand {
    Get {
        key: String,
        resp: Responder<Option<String>>,
    },
    Set {
        key: String,
        val: String,
        resp: Responder<()>,
    },
}

type Responder<T> = oneshot::Sender<AnyResult<T>>;

pub async fn new_redis_sender(mut redis_client: RedisClient) -> Sender<RedisCommand> {
    let (sender, mut receiver) = mpsc::channel::<RedisCommand>(32);
    tokio::spawn(async move {
        // Start receiving messages
        while let Some(cmd) = receiver.recv().await {
            match cmd {
                RedisCommand::Get { key, resp } => {
                    let _ = resp.send(redis_client.get(&key));
                }
                RedisCommand::Set { key, val, resp } => {
                    let _ = resp.send(redis_client.set(&key, &val));
                }
            }
        }
        Ok::<(), Report>(())
    });
    sender
}

pub trait RedisCommands {
    async fn get(&self, key: &str) -> AnyResult<Option<String>>;
    async fn set(&self, key: &str, value: &str) -> AnyResult<()>;
}

impl RedisCommands for Sender<RedisCommand> {
    async fn get(&self, key: &str) -> AnyResult<Option<String>> {
        let (sender, receiver) = oneshot::channel::<AnyResult<Option<String>>>();
        self.send(RedisCommand::Get {
            key: key.to_owned(),
            resp: sender,
        })
        .await?;
        receiver.await?
    }

    async fn set(&self, key: &str, value: &str) -> AnyResult<()> {
        let (sender, receiver) = oneshot::channel::<AnyResult<()>>();
        self.send(RedisCommand::Set {
            key: key.to_owned(),
            val: value.to_owned(),
            resp: sender,
        })
        .await?;
        receiver.await?
    }
}
