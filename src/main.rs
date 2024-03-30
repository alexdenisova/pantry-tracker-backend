#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
use crate::server::AppState;
use clap::Parser;
use color_eyre::Result as AnyResult;
use dotenvy::dotenv;
use migrations::{Migrator, MigratorTrait};
use sea_orm::Database;
use server::Server;
use settings::{Cli, Commands};

mod database;
mod server;
mod settings;

#[tokio::main]
async fn main() -> AnyResult<()> {
    dotenv().ok();
    let cli = Cli::parse();
    cli.setup_logging()?;

    let db_connection = Database::connect(cli.database.url).await?;
    match cli.command {
        Commands::Run(args) => {
            // let dao = DatabaseClient::new(db_connection);
            let state = AppState::new(db_connection);
            let server = Server::new(state);

            server.run(args.socket).await.unwrap();
        }
        Commands::Migrate => Migrator::up(&db_connection, None).await?,
    }
    Ok(())
}
