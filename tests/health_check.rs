use sqlx::{Connection, Executor, PgConnection, PgPool, Pool, Postgres};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration_settings, DatabaseSettings};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;

// Ensure the subscriber is only instantiated once, even if called multiple times:
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::sink); 
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    // Initialize the subscriber:
    Lazy::force(&TRACING);
    
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let address =  format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration_settings().expect("Failed to read configuration settings.");
    // Randomize the database name:
    configuration.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_db(&configuration.database).await;

    let server = run(listener, db_pool.clone()).expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool,
    }
}

async fn configure_db(config: &DatabaseSettings) -> Pool<Postgres> {
    // Spin up the database
    let mut connection = PgConnection::connect(&config.db_connection_string_without_db_name())
        .await
        .expect("Failed to connect to Postgres.");

    connection.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate data:
    let connection_pool = PgPool::connect(&config.db_connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed migrating the database!");

    connection_pool
}

#[tokio::test]
async fn health_check_test() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        // The "&" is used to prevent duplication of the variable, instead uses the reference. 
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let request_body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(request_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let subscription_saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

        assert_eq!(subscription_saved.email, "ursula_le_guin@gmail.com");
        assert_eq!(subscription_saved.name, "le guin");

}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let request_bodys = vec![
       ("name=le%20guin", "missing the email"),
       ("&email=ursula_le_guin%40gmail.com", "missing the name"),
       ("", "missing both email and name")
    ];

    for (invalid_body, message) in request_bodys {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            response.status().as_u16(),
            400,
            "Request should have failed with the payload {}",
            message
        )
    }
}
