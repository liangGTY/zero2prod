use std::io::Error;
use std::net::TcpListener;
use zero2prod::run;

#[tokio::test]
async fn health_check_succeeds() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/health_check",address))
        .send()
        .await
        .expect("Failed to send request to http://127.0.0.1:8000/health_check");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(run(listener).expect("Failed to bind address"));
    format!("127.0.0.1:{}", port)
}
