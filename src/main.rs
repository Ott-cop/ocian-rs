use actix_cors::Cors;
use actix_web::{http::{header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE}, KeepAlive}, web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env::{self, VarError}, time::Duration};

mod api;
mod models;
use api::api::{send_contact_us, send_proposal, send_support, send_work_with_us};

#[derive(Clone)]
struct AppState {
    pool: PgPool
}

#[actix_web::main]
async fn main() {
    
    dotenv::dotenv().ok();

    let db_url: Result<String, VarError> = env::var("DATABASE_URL");

    let pool = match PgPoolOptions::new().connect(db_url.unwrap().as_str()).await {
        Ok(pool) => { 
            println!("[+] Stabilized connection to the server!");
            pool
        },
        Err(err) => {
            panic!("[!] Error found: {}", err);
        }
    };

    let _ = sqlx::migrate!("./migrations").run(&pool).await; 


    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_origin("https://ocian.vercel.app")
            .allowed_headers(vec![CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN])
            .allowed_methods(vec!["POST"])
            .max_age(3600);
        App::new()
        .app_data(web::Data::new(AppState { pool: pool.clone() }))
            .wrap(cors)
            .route("/send_proposal", web::post().to(send_proposal))
            .route("/send_contact_us", web::post().to(send_contact_us))
            .route("/send_work_with_us", web::post().to(send_work_with_us))
            .route("/send_support", web::post().to(send_support))
    })
    .bind(("0.0.0.0", 8080))
    .unwrap()
    .shutdown_timeout(5)
    .keep_alive(KeepAlive::Timeout(Duration::from_millis(1000)))
    .run()
    .await
    .unwrap();
}