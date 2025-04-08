use actix_cors::Cors;
use actix_web::{http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE}, web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env::{self}, sync::Mutex};

mod api;
mod models;
use api::api::{send_contact_us, send_proposal, send_support, send_work_with_us};

struct AppState {
    pool: Mutex<PgPool>
}

#[actix_web::main]
async fn main() {
    dotenv::dotenv().ok();

    let db_user = env::var("DB_USER").expect("Enter the DB_USER environment variable correctly");
    let db_password = env::var("DB_PASSWORD").expect("Enter the DB_PASSWORD environment variable correctly");
    let db_name = env::var("DATABASE_NAME").expect("Enter the DATABASE_NAME environment variable correctly");
    let db_server = env::var("DATABASE_HOST").expect("Enter the DATABASE_HOST environment variable correctly");
    let db_port = env::var("DATABASE_PORT").expect("Enter the DATABASE_PORT environment variable correctly");

    let conn = format!("postgres://{db_user}:{db_password}@{db_server}:{db_port}/{db_name}");

    let pool = match PgPoolOptions::new().connect(&conn).await {
        Ok(pool) => { 
            println!("[+] Stabilized connection to the server!");
            pool
        },
        Err(err) => {
            panic!("[!] Error found: {}", err);
        }
    };

    let _ = sqlx::migrate!("./migrations").run(&pool).await; 
    let app_state = web::Data::new(AppState { pool: Mutex::new(pool) });


    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_origin("https://ocian.vercel.app")
            .allowed_headers(vec![CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN])
            .allowed_methods(vec!["POST"])
            .max_age(3600);
        App::new()
        .app_data(app_state.clone())
            .wrap(cors)
            .route("/send_proposal", web::post().to(send_proposal))
            .route("/send_contact_us", web::post().to(send_contact_us))
            .route("/send_work_with_us", web::post().to(send_work_with_us))
            .route("/send_support", web::post().to(send_support))
    })
    .bind(("0.0.0.0", 8080))
    .unwrap()
    .run()
    .await
    .unwrap();
}