use sqlx::PgPool;
use zero2prod::{configuration::get_configuration_settings, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration_settings().expect("Failed to read configuration settings."); 
    let connection_string = configuration.database.db_connection_string();
    let db = PgPool::connect(&connection_string).await.expect("Failed to connect to Postgres.");

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = std::net::TcpListener::bind(address)?;
    run(listener, db)?.await
}
