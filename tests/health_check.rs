use std::net::TcpListener;

use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool, Pool, Postgres};
use uuid::Uuid;

use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

struct TestApp {
    http_url: String,
    db_pool: Pool<Postgres>,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    init_subscriber(get_subscriber("test".into(), "info".into()));
});

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = get_configuration().expect("Failed to get configuration");

    configuration.database.database_name = Uuid::new_v4().to_string();

    let pool = configuration_database(&configuration.database).await;

    tokio::spawn(run(listener, pool.clone()).expect("Failed to bind address"));

    TestApp {
        http_url: format!("http://127.0.0.1:{}", port),
        db_pool: pool.clone(),
    }
}

async fn configuration_database(config: &DatabaseSettings) -> Pool<Postgres> {
    let mut connection = PgConnection::connect(
        &config
            .connection_string_without_database_name()
            .expose_secret(),
    )
    .await
    .expect("Failed to connect to database");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    pool
}

#[tokio::test]
async fn health_check_succeeds() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", test_app.http_url))
        .send()
        .await
        .expect("Failed to send request to /health_check");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();
    let body = "name=tom&email=tom@tom.com";
    let response = client
        .post(format!("{}/subscriptions", test_app.http_url))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request to /subscriptions");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "tom@tom.com");
    assert_eq!(saved.name, "tom");
}

#[tokio::test]
async fn subscribe_returns_a_400_for_data_missing() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let body_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error_msg) in body_cases {
        let response = client
            .post(format!("{}/subscriptions", test_app.http_url))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to send request to /subscriptions");

        assert_eq!(
            400,
            response.status().as_u16(),
            "Failed to send request to /subscriptions {}.",
            error_msg
        );
    }
}
