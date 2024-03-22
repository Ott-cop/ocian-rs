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

struct Database<'a> {
    app_state: web::Data<AppState>,
    table: &'a str
}

impl Database<'_> {
    fn new<'a>(table: &'a str, app_state: web::Data<AppState>) -> Database<'_> {
        Database {
            app_state,
            table
        }
    }
    async fn insert(self, req: web::Json<User>) -> HttpResponse {
        let query = format!("INSERT INTO {} (name, email, phone, subject, message) VALUES ($1, $2, $3, $4, $5)", self.table);
        match sqlx::query(query.as_str())
        .bind(&req.name)
        .bind(&req.email)
        .bind(&req.phone)
        .bind(&req.subject)
        .bind(&req.message)
        .execute(&*self.app_state.pool.lock().unwrap()).await {
            Ok(_) => HttpResponse::Ok().json("The request has been sended!"),
            Err(err) => HttpResponse::BadRequest().json(err.to_string())
        }
    }
}

struct FileForm {
    file: Vec<u8>,
    file_type: String
}

impl FileForm {
    fn new(temp_file: &mut TempFile) -> Result<FileForm, HttpResponse> {
        let mut file: Vec<u8> = vec![];
        let _ = temp_file.file.read_to_end(&mut file);

        let file_type: Result<String, HttpResponse> = match temp_file.content_type.clone().unwrap().to_string() {
            ftype if ftype == String::from("application/pdf") => Ok(String::from("application/pdf")),
            ftype if ftype == String::from("application/msword") => Ok(String::from("application/msword")),
            ftype if ftype == String::from("application/vnd.openxmlformats-officedocument.wordprocessingml.document") => Ok(String::from("application/vnd.openxmlformats-officedocument.wordprocessingml.document")),
            _ => Err(HttpResponse::BadRequest().json("The file format is invalid."))
        };
    
        if let Err(err) = file_type {
            return Err(err);
        }
        let file_type = file_type.unwrap();

        Ok(FileForm {
            file,
            file_type
        })
    }
}

struct Mail {
    host: String,
    username: String,
    password: String,
    from_email: String,
    to_email: String
}

impl Mail {
    fn env_init() -> Mail {
        let from_email = env::var("FROM_EMAIL").expect("FROM_EMAIL not found!");
        let host = env::var("HOST").expect("HOST not found!");
        let to_email = env::var("TO_EMAIL").expect("TO_EMAIL not found!");
        let username = env::var("USERNAME").expect("USERNAME not found!");
        let password = env::var("PASSWORD").expect("PASSWORD not found!");

        Mail {
            host,
            username,
            password,
            from_email,
            to_email
        }
    }

    fn new(&self, form: Curriculum) -> Result<(SmtpTransport, Message), HttpResponse> {

        if let Err(err) = validate(&form.name.0, &form.email.0, &form.phone.0, None, &form.message.0, Some(&form.file)) {
            return Err(HttpResponse::BadRequest().json(err));
        }
        
        let mut formfile = form.file;
        let file = FileForm::new(&mut formfile);

        if let Err(err) = file {
            return Err(err);
        }
        let file = file.unwrap();

        let content_type = ContentType::parse(&file.file_type).unwrap();

        let attachment = Attachment::new(formfile.file_name.clone().unwrap()).body(file.file, content_type);

        let body = format!("\nNome: {}\nEmail: {}\nTelefone: {}\n\nMensagem: {}", form.name.0, form.email.0, form.phone.0, form.message.0);

        let message = Message::builder()
            .from(self.from_email.parse::<Mailbox>().unwrap())
            .to(self.to_email.parse::<Mailbox>().unwrap())
            .multipart(
                MultiPart::mixed()
                    .singlepart(SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(body.parse::<String>().unwrap())
                    )
                    .singlepart(attachment)
            ).unwrap();

        let mailer = SmtpTransport::starttls_relay(&self.host)
            .unwrap()
            .credentials(Credentials::new(self.username.to_owned(), self.password.to_owned()))
            .port(587)
            .build();

        Ok((mailer, message))

    }
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

pub async fn send_proposal(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {
    
    if let Err(err) = validate(&req.name, &req.email, &req.phone, Some(&req.subject), &req.message, None) { 
        return HttpResponse::BadRequest().json(err);
    } 

    let db = Database::new("proposal", app_state);
    db.insert(req).await
}

pub async fn send_contact_us(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {

    if let Err(err) = validate(&req.name, &req.email, &req.phone, Some(&req.subject), &req.message, None) { 
        return HttpResponse::BadRequest().json(err);
    } 

    let db = Database::new("contact_us", app_state);
    db.insert(req).await

}

pub async fn send_work_with_us(MultipartForm(form): MultipartForm<Curriculum>) -> impl Responder {
    
    let mail = Mail::env_init();
    let email = mail.new(form);

    if let Err(err_mail) = email{
        return err_mail;
    }

    let (mailer, message) = email.unwrap();

    match mailer.send(&message) {
        Ok(_) => HttpResponse::Ok().json("The message has been sended!"),
        Err(err) => HttpResponse::BadGateway().json(err.to_string())
    }
    
}

pub async fn send_support(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {
    
    if let Err(err) = validate(&req.name, &req.email, &req.phone, Some(&req.subject), &req.message, None) { 
        return HttpResponse::BadRequest().json(err);
    } 

    let db = Database::new("support", app_state);
    db.insert(req).await
}


