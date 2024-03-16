use std::env::{self};
use std::io::Read;
use actix_multipart::form::MultipartForm;
use actix_multipart::form::{tempfile::TempFile, text::Text};
use actix_web::{web, HttpResponse, Responder};
use lettre::message::header::{self, ContentType};
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::AppState;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct User {
    name: String,
    email: String,
    phone: String,
    subject: String,
    message: String,
}

#[derive(MultipartForm)]
pub struct Curriculum {
    name: Text<String>,
    email: Text<String>,
    phone: Text<String>,
    file: TempFile,
    message: Text<String>
}

#[derive(serde::Serialize, Debug)]
struct Error<'a> {
    message: &'a str,
    error: Vec<&'a str>
}


fn validate(name: &String, email: &String, phone: &String, subject: Option<&String>, message: &String, file: Option<&TempFile>) -> Result<(), Error<'static>> {

    let mut error: Vec<&str> = vec![];

    if name.len() < 6 || name.len() > 50 {
        error.push("name");
    } if email.len() < 8 || email.len() > 50 {
        error.push("email");
    } if phone.len() < 8 || phone.len() > 20 {
        error.push("phone");
    } if subject.is_some() {
        if subject.unwrap().len() < 6 || subject.unwrap().len() > 50 {
            error.push("subject");
        }
    } if message.len() < 6 || message.len() > 500 {
        error.push("message");
    } if file.is_some() {
        let file = file.unwrap();

        if file.file_name.is_none() {
            error.push("file");
        } 
    }

    let error_json = Error {
        message: "The following fields are incorrect:",
        error: error.clone()
    };

    if error.len() > 0 {
        return Err(error_json);
    }

    Ok(())
}

fn make_email(host: String, username: String, password: String, from_email: String, to_email: String, name: &String, email: &String, phone: &String, message: &String, filebody: Vec<u8>, filename: String, type_value: String) -> (SmtpTransport, Message) {
  

    let content_type = ContentType::parse(type_value.as_str()).unwrap();

    let attachment = Attachment::new(filename).body(filebody, content_type);


    let body = format!("\nNome: {}\nEmail: {}\nTelefone: {}\n\nMensagem: {}", name, email, phone, message);
    let message = Message::builder()
        .from(from_email.parse::<Mailbox>().unwrap())
        .to(to_email.parse::<Mailbox>().unwrap())
        .multipart(
            MultiPart::mixed()
                .singlepart(SinglePart::builder()
                    .header(header::ContentType::TEXT_PLAIN)
                    .body(body.parse::<String>().unwrap())
                )
                .singlepart(attachment)
        ).unwrap();
    
    let mailer = SmtpTransport::starttls_relay(&host)
        .unwrap()
        .credentials(Credentials::new(username.to_owned(), password.to_owned()))
        .port(587)
        .build();

    (mailer, message)
}

pub async fn send_proposal(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {
    
    if let Err(err) = validate(&req.name, &req.email, &req.phone, Some(&req.subject), &req.message, None) { 
        return HttpResponse::BadRequest().json(err);
    } 
    
    match sqlx::query("INSERT INTO proposal (name, email, phone, subject, message) VALUES ($1, $2, $3, $4, $5)")
    .bind(&req.name)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.subject)
    .bind(&req.message)
    .execute(&*app_state.pool.read().unwrap()).await {
        Ok(_) => HttpResponse::Ok().json("The request has been sended!"),
        Err(err) => HttpResponse::BadRequest().json(err.to_string())
    }
}

pub async fn send_contact_us(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {

    if let Err(err) = validate(&req.name, &req.email, &req.phone, Some(&req.subject), &req.message, None) { 
        return HttpResponse::BadRequest().json(err);
    } 

    match sqlx::query("INSERT INTO contact_us (name, email, phone, subject, message) VALUES ($1, $2, $3, $4, $5)")
    .bind(&req.name)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.subject)
    .bind(&req.message)
    .execute(&*app_state.pool.read().unwrap()).await {
        Ok(_) => HttpResponse::Ok().json("The request has been sended!"),
        Err(err) => HttpResponse::BadRequest().json(err.to_string())
    }
}

pub async fn send_work_with_us(MultipartForm(mut form): MultipartForm<Curriculum>) -> impl Responder {
    
    let from_email: String = env::var("FROM_EMAIL").expect("FROM_EMAIL not found!");
    let host: String = env::var("HOST").expect("HOST not found!");
    let to_email = env::var("TO_EMAIL").expect("TO_EMAIL not found!");
    let username = env::var("USERNAME").expect("USERNAME not found!");
    let password = env::var("PASSWORD").expect("PASSWORD not found!");

    if let Err(err) = validate(&form.name.0, &form.email.0, &form.phone.0, None, &form.message.0, Some(&form.file)) {
        return HttpResponse::BadRequest().json(err);
    }
    let mut file: Vec<u8> = vec![];
    let _ = form.file.file.read_to_end(&mut file);
    
    let type_value: Result<String, HttpResponse> = match form.file.content_type.unwrap().to_string() {
        ftype if ftype == String::from("application/pdf") => Ok(String::from("application/pdf")),
        ftype if ftype == String::from("application/msword") => Ok(String::from("application/msword")),
        ftype if ftype == String::from("application/vnd.openxmlformats-officedocument.wordprocessingml.document") => Ok(String::from("application/vnd.openxmlformats-officedocument.wordprocessingml.document")),
        _ => Err(HttpResponse::BadRequest().json("The file format is invalid."))
    };

    if let Err(err) = type_value {
        return err;
    }
    
    let (mailer, message) = make_email(host, username, password, from_email, to_email, &form.name.0, &form.email.0, &form.phone.0, &form.message.0, file, form.file.file_name.unwrap(), type_value.unwrap());

    match mailer.send(&message) {
        Ok(_) => HttpResponse::Ok().json("The message has been sended!"),
        Err(err) => HttpResponse::BadGateway().json(err.to_string())
    }
    
}

pub async fn send_support(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {
    
    if let Err(err) = validate(&req.name, &req.email, &req.phone, Some(&req.subject), &req.message, None) { 
        return HttpResponse::BadRequest().json(err);
    } 

    match sqlx::query("INSERT INTO support (name, email, phone, subject, message) VALUES ($1, $2, $3, $4, $5)")
    .bind(&req.name)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.subject)
    .bind(&req.message)
    .execute(&*app_state.pool.read().unwrap()).await {
        Ok(_) => HttpResponse::Ok().json("The request has been sended!"),
        Err(err) => HttpResponse::BadRequest().json(err.to_string())
    }
}


