use color_eyre::Report as AnyError;
use redis::{Client, Commands, Connection};
use thiserror::Error;
use tokio::sync::{
    mpsc::{self, error::SendError, Sender},
    oneshot::{self, error::RecvError},
};

pub type RedisResult<T> = Result<T, RedisError>;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Redis error: {error}")]
    Redis { error: AnyError },
    #[error("Error in channel: {error}")]
    Channel { error: AnyError },
}

impl From<redis::RedisError> for RedisError {
    fn from(value: redis::RedisError) -> Self {
        RedisError::Redis {
            error: value.into(),
        }
    }
}

impl From<SendError<RedisCommand>> for RedisError {
    fn from(value: SendError<RedisCommand>) -> Self {
        RedisError::Channel {
            error: value.into(),
        }
    }
}

impl From<RecvError> for RedisError {
    fn from(value: RecvError) -> Self {
        RedisError::Channel {
            error: value.into(),
        }
    }
}

pub struct RedisClient {
    connection: Connection,
}

impl RedisClient {
    pub fn new(url: &str) -> RedisResult<Self> {
        Ok(RedisClient {
            connection: Client::open(url)?.get_connection()?,
        })
    }

    pub fn get(&mut self, key: &str) -> RedisResult<Option<String>> {
        self.connection.get(key).map_err(Into::into)
    }

    pub fn set(&mut self, key: &str, value: &str, ttl_days: Option<u16>) -> RedisResult<()> {
        self.connection.set(key, value)?;
        if let Some(days) = ttl_days {
            self.connection
                .expire(key, i64::from(days) * 24 * 60 * 60)?; // TODO: make this a cmd arg
        }
        Ok(())
    }

    pub fn delete(&mut self, key: &str) -> RedisResult<()> {
        self.connection.del(key).map_err(Into::into)
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
        ttl_days: Option<u16>,
        resp: Responder<()>,
    },
    Delete {
        key: String,
        resp: Responder<()>,
    },
}

type Responder<T> = oneshot::Sender<RedisResult<T>>;

pub async fn new_redis_sender(mut redis_client: RedisClient) -> Sender<RedisCommand> {
    let (sender, mut receiver) = mpsc::channel::<RedisCommand>(32); //TODO: make this a constant
    tokio::spawn(async move {
        // Start receiving messages
        while let Some(cmd) = receiver.recv().await {
            match cmd {
                RedisCommand::Get { key, resp } => {
                    let _ = resp.send(redis_client.get(&key));
                }
                RedisCommand::Set {
                    key,
                    val,
                    ttl_days: expire_days,
                    resp,
                } => {
                    let _ = resp.send(redis_client.set(&key, &val, expire_days));
                }
                RedisCommand::Delete { key, resp } => {
                    let _ = resp.send(redis_client.delete(&key));
                }
            }
        }
        Ok::<(), AnyError>(())
    });
    sender
}

pub trait RedisCommands {
    async fn get(&self, key: &str) -> RedisResult<Option<String>>;
    async fn set(&self, key: &str, value: &str, expire_days: Option<u16>) -> RedisResult<()>;
    async fn delete(&self, key: &str) -> RedisResult<()>;
}

impl RedisCommands for Sender<RedisCommand> {
    async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        let (sender, receiver) = oneshot::channel::<RedisResult<Option<String>>>();
        self.send(RedisCommand::Get {
            key: key.to_owned(),
            resp: sender,
        })
        .await?;
        receiver.await?
    }

    async fn set(&self, key: &str, value: &str, expire_days: Option<u16>) -> RedisResult<()> {
        let (sender, receiver) = oneshot::channel::<RedisResult<()>>();
        self.send(RedisCommand::Set {
            key: key.to_owned(),
            val: value.to_owned(),
            ttl_days: expire_days,
            resp: sender,
        })
        .await?;
        receiver.await?
    }

    async fn delete(&self, key: &str) -> RedisResult<()> {
        let (sender, receiver) = oneshot::channel::<RedisResult<()>>();
        self.send(RedisCommand::Delete {
            key: key.to_owned(),
            resp: sender,
        })
        .await?;
        receiver.await?
    }
}
