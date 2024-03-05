use std::net::TcpListener;
use actix_web::{dev::Server, web, App, HttpServer, middleware::Logger};
use sqlx::PgPool;

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, db: PgPool) -> Result<Server, std::io::Error> {
    let listener_clone = listener.try_clone()?;
    let db = web::Data::new(db);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db.clone())
    })
    .listen(listener_clone)?
    .run();

    log::info!("Server connected, listening for requests at: {} ", listener.local_addr()?);

    Ok(server)
}