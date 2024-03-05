use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, db: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subsriber!",
        %request_id,
        request_email = %form.email,
        request_name = %form.name,
    );

    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!(
        "Saving new subscriber details in the database"
    );

    match sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
        .execute(db.as_ref())
        .instrument(query_span)
    .await {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            // :? gives a more detailed error message
            tracing::error!("Request ID: {} - Failed to execute query: {:?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        },
    }
}