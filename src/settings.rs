use chrono::prelude::Local;
use clap::{Args, Parser, Subcommand};
use color_eyre::Result as AnyResult;
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;
use log::LevelFilter;
use tokio::sync::mpsc::Sender;
use url::Url;

use crate::redis::{new_redis_sender, RedisClient, RedisCommand};

const DEFAULT_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432/postgres";
const DEFAULT_REDIS_URL: &str = "redis://localhost:6379/0";
const DEFAULT_SOCKET: &str = "localhost:8080";

#[derive(Debug, Parser)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[command(flatten)]
    pub database: DatabaseArguments,
    #[command(flatten)]
    redis: RedisArguments,
    /// Set debug log level
    #[arg(long, short = 'd', default_value = "false", env = "APP__DEBUG")]
    debug: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Run HTTP server")]
    Run(RunArgs),
    #[command(about = "Run database migrations and exit")]
    Migrate,
    #[command(about = "Fill database with test data")]
    Test,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    /// App socket address (<host>:<port>)
    #[arg(
        long = "socket",
        env = "APP__SOCKET",
        default_value = DEFAULT_SOCKET
    )]
    pub socket: String,
}

#[derive(Debug, Args)]
pub struct DatabaseArguments {
    /// Database URL
    #[arg(
        long = "db-url",
        id = "db-url",
        env = "APP__DATABASE_URL",
        default_value = DEFAULT_DATABASE_URL,
        global = true
    )]
    pub url: Url,
}

#[derive(Debug, Args)]
pub struct RedisArguments {
    /// Redis URL
    #[arg(long = "redis-url", id = "redis-url", env = "APP__REDIS_URL", default_value = DEFAULT_REDIS_URL,)]
    url: Url,
}

impl RedisArguments {
    pub fn client(&self) -> AnyResult<RedisClient> {
        RedisClient::new(
            &self.url.to_string()
        )
    }
}

impl Cli {
    pub fn setup_logging(&self) -> AnyResult<()> {
        let colors = ColoredLevelConfig::new()
            .debug(Color::BrightBlack)
            .info(Color::BrightGreen)
            .warn(Color::BrightYellow)
            .error(Color::BrightRed);
        if self.debug {
            Dispatch::new()
                .level(LevelFilter::Debug)
                .level_for("html5ever", LevelFilter::Off)
                .level_for("selectors", LevelFilter::Off)
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "{} [{}] {}: {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S MSK"),
                        colors.color(record.level()),
                        record.target(),
                        message,
                    ));
                })
        } else {
            Dispatch::new()
                .level(LevelFilter::Info)
                .level_for("sqlx::query", LevelFilter::Off)
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "{} [{}] {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S MSK"),
                        colors.color(record.level()),
                        message,
                    ));
                })
        }
        .chain(std::io::stderr())
        .apply()?;
        Ok(())
    }
    pub async fn redis_sender(&self) -> AnyResult<Sender<RedisCommand>> {
        let redis_client = self.redis.client()?;
        Ok(new_redis_sender(redis_client).await)
    }
}
