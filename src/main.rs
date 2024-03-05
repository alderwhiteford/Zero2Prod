use std::net::TcpListener;
use sqlx::PgPool;
use zero2prod::configuration::get_configuration_settings;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize the subscriber:
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
        
    let configuration = get_configuration_settings().expect("Failed to read configuration settings."); 
    let connection_string = configuration.database.db_connection_string();
    log::info!("Connecting to Postgres...");

    let db = PgPool::connect(&connection_string).await.expect("Failed to connect to Postgres.");
    log::info!("Successfully connected to Postgres: {}", connection_string);

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, db)?.await
}
