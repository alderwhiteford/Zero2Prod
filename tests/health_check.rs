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

fn spawn_app() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to a random port");

    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
