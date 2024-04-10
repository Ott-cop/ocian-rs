use actix_multipart::form::MultipartForm;
use actix_web::{web, HttpResponse, Responder};
use lettre::Transport;
use validator::Validate;

use crate::AppState;
use crate::models::user::User;
use crate::models::curriculum::{Curriculum, RecvCurriculum};
use crate::models::database::{Database, Response};
use crate::models::email::Mail;


const RESPONSE: Response<'static> = Response { response: "The message has been sended!" };

pub async fn send_proposal(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {
    
    if let Err(err) = req.validate() {
        return HttpResponse::BadRequest().json(err);
    } 

    let db = Database::table("proposal", app_state);
    if let Err(err) = db.insert(req).await {
        return HttpResponse::BadRequest().json(err.to_string());
    }

    HttpResponse::Ok().json(RESPONSE)
}

pub async fn send_contact_us(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {

    if let Err(err) = req.validate() {
        return HttpResponse::BadRequest().json(err);
    }

    let db = Database::table("contact_us", app_state);
    if let Err(err) = db.insert(req).await {
        return HttpResponse::BadRequest().json(err.to_string());
    }

    HttpResponse::Ok().json(RESPONSE)
}

pub async fn send_work_with_us(MultipartForm(form): MultipartForm<RecvCurriculum>) -> impl Responder {

    let form = Curriculum::new(form);

    if let Err(err) = form.validate() {
        return HttpResponse::BadRequest().json(err);
    }

    let mail = Mail::env_init();

    let email = mail.new(form);

    if let Err(err_mail) = email {
        return HttpResponse::BadRequest().json(Response { response: &err_mail });
    }

    let (mailer, message) = email.unwrap();

    match mailer.send(&message) {
        Ok(_) => HttpResponse::Ok().json(RESPONSE),
        Err(err) => HttpResponse::BadRequest().json(Response { response: &err.to_string() })
    }
    
}

pub async fn send_support(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {
    
    if let Err(err) = req.validate() {
        return HttpResponse::BadRequest().json(err);
    }

    let db = Database::table("support", app_state);
    if let Err(err) = db.insert(req).await {
        return HttpResponse::BadRequest().json(err.to_string());
    }

    HttpResponse::Ok().json(RESPONSE)
}


