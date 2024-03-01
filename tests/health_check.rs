use sqlx::PgPool;
use zero2prod::{configuration::get_configuration_settings, startup::run};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let address =  format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration_settings().expect("Failed to read configuration settings.");
    let connection_string = configuration.database.db_connection_string();
    let db = PgPool::connect(&connection_string).await.expect("Failed to connect to Postgres.");

    let server = run(listener, db.clone()).expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    TestApp {
        address: address,
        db_pool: db,
    }
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
