use sqlx::{PgConnection, Connection};
use zero2prod::{configuration::get_configuration_settings, routes::FormData, startup::run};

fn spawn_app() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to a random port");

    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_test() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        // The "&" is used to prevent duplication of the variable, instead uses the reference. 
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let address = spawn_app();
    let configuration = get_configuration_settings().expect("Failed to read configuration settings.");
    let connection_string = configuration.database.db_connection_string();
    let mut connection = 
        PgConnection::connect(&connection_string)
            .await
            .expect("Failed to connect to Postgres.");

    let client = reqwest::Client::new();

    let request_body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(request_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let subscription_saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

        assert_eq!(subscription_saved.email, "ursula_le_guin@gmail.com");
        assert_eq!(subscription_saved.name, "le guin");

}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let request_bodys = vec![
       ("name=le%20guin", "missing the email"),
       ("&email=ursula_le_guin%40gmail.com", "missing the name"),
       ("", "missing both email and name")
    ];

    for (invalid_body, message) in request_bodys {
        let response = client
            .post(&format!("{}/subscriptions", address))
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
