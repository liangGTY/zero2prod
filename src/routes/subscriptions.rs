use actix_web::{HttpResponse, web};
use actix_web::web::Form;
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscriptions(form: Form<FormData>, _connection: web::Data<PgPool>) -> HttpResponse {
    let result = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id ,email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(_connection.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
