use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to load configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");

    run(listener, pool)?.await
}
