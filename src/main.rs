use actix_cors::Cors;
use actix_web::{http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE}, web::{self}, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env::{self, temp_dir, VarError};

mod api;
use api::{send_contact_us, send_proposal, send_support, send_work_with_us};

#[derive(Clone)]
struct AppState {
    pool: PgPool
}

#[actix_web::main]
async fn main() {
    dotenv::dotenv().ok();

    let db_url: Result<String, VarError> = env::var("DATABASE_URL");

    let pool = PgPoolOptions::new().connect(db_url.unwrap().as_str()).await;
    match &pool {
        Ok(_) => println!("[+] Stabilized connection to the server!"),
        Err(err) => println!("[!] Error found: {}", err)
    }
    let pool = pool.unwrap();

    let _ = sqlx::migrate!("./migrations").run(&pool).await; 

    let app_state = AppState { pool: pool.clone() };

    let _ = std::process::Command::new("mkdir")
        .arg(format!("{}/temp_curriculum", temp_dir().display()))
        .spawn()
        .expect("[!] Existing folder...")
        .wait();
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("https://ocian.vercel.app")
            .allowed_headers(vec![CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN])
            .allowed_methods(vec!["POST"])
            .max_age(3600);
        App::new()
        .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .route("/send_proposal", web::post().to(send_proposal))
            .route("/send_contact_us", web::post().to(send_contact_us))
            .route("/send_work_with_us", web::post().to(send_work_with_us))
            .route("/send_support", web::post().to(send_support))

    }).bind(("0.0.0.0", 8080)).unwrap().run().await.unwrap();


}