use actix_web::web;
use serde::Serialize;

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

impl Database<'_> {
    pub fn table<'a>(table: &'a str, app_state: web::Data<AppState>) -> Database<'_> {
        Database {
            app_state,
            table
        }
    }
    pub async fn insert(self, req: web::Json<User>) -> Result<(), sqlx::Error> {
        
        let query = format!("INSERT INTO {} (name, email, phone, subject, message) VALUES ($1, $2, $3, $4, $5)", self.table);
        if let Err(err) = sqlx::query(&query)
            .bind(&req.name)
            .bind(&req.email)
            .bind(&req.phone)
            .bind(&req.subject)
            .bind(&req.message)
            .execute(&self.app_state.pool).await {
                return Err(err);
        }
        Ok(())
    }
}