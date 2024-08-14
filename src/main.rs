#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
use crate::server::AppState;
use clap::Parser;
use color_eyre::Result as AnyResult;
use dao::db_client::DatabaseClient;
use dotenvy::dotenv;
use migrations::{Migrator, MigratorTrait};
use sea_orm::*;
use sea_orm_migration::prelude::*;
use server::Server;
use settings::{Cli, Commands};

mod dao;
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
            let dao = DatabaseClient::new(db_connection);
            let state = AppState::new(dao);
            let server = Server::new(state);

            server.run(args.socket).await.unwrap();
        }
        Commands::Migrate => Migrator::up(&db_connection, None).await?,
    }
    Ok(())
}
