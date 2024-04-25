#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
mod database;
mod redis;
mod server;
mod settings;
mod test;

use clap::Parser;
use color_eyre::Result as AnyResult;
use dotenvy::dotenv;
use migrations::{Migrator, MigratorTrait};
use sea_orm::Database;
use server::Server;

use crate::server::AppState;
use settings::{Cli, Commands};

#[tokio::main]
async fn main() -> AnyResult<()> {
    dotenv().ok();
    let cli = Cli::parse();
    cli.setup_logging()?;

    let redis_sender = cli.redis_sender().await?;
    let db_connection = Database::connect(cli.database.url).await?;

    match cli.command {
        Commands::Run(args) => {
            let state = AppState::new(db_connection, redis_sender);
            let server = Server::new(state);

            log::info!("Server listening on {}", args.socket);
            server.run(args.socket).await.unwrap();
        }
        Commands::Migrate => Migrator::up(&db_connection, None).await?,
        Commands::Test => {
            let client = database::DBClient::new(db_connection);
            test::migrate_test_data(client).await?;
        }
    }
    Ok(())
}
