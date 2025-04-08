use actix_web::web;
use serde::Serialize;
use sqlx::Postgres;

use crate::AppState;

use super::user::User;

#[derive(Serialize)]
pub struct Response<'a> {
    pub response: &'a str
}

pub struct Database<'a> {
    app_state: web::Data<AppState>,
    table: &'a str
}

impl<'a> Database<'a> {
    pub fn table(table: &'a str, app_state: web::Data<AppState>) -> Self {
        Database {
            app_state,
            table
        }
    }
    pub async fn insert(self, req: web::Json<User>) -> Result<(), sqlx::Error> {
        let query = format!("INSERT INTO {} (name, email, phone, subject, message) VALUES ($1, $2, $3, $4, $5)", self.table);
        let state = self.app_state.pool.lock().unwrap();
        if let Err(err) = sqlx::query::<Postgres>(&query)
            .bind(&req.name)
            .bind(&req.email)
            .bind(&req.phone)
            .bind(&req.subject)
            .bind(&req.message)
            .execute(&*state).await {
                return Err(err);
        }
  
        Ok(())
    }
}