use std::env::{self};
use std::io::Read;
use actix_multipart::form::MultipartForm;
use actix_multipart::form::{tempfile::TempFile, text::Text};
use actix_web::{web, HttpResponse, Responder};
use lettre::message::header::{self, ContentType};
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use validator::Validate;

use crate::AppState;

#[derive(serde::Deserialize, serde::Serialize, Validate)]
pub struct User {
    #[validate(length(min = 5, max = 50))]
    name: String,

    #[validate(length(min = 7, max = 50), email)]
    email: String,

    #[validate(length(min = 7, max = 20))]
    phone: String,

    #[validate(length(min = 5, max = 50))]
    subject: String,

    #[validate(length(min = 5, max = 500))]
    message: String,
}


#[derive(MultipartForm)]
pub struct RecvCurriculum {
    name: Text<String>,
    email: Text<String>,
    phone: Text<String>,
    file: TempFile,
    message: Text<String>
}

#[derive(Validate)]
struct Curriculum {
    #[validate(length(min = 5, max = 50))]
    name: String,

    #[validate(length(min = 7, max = 50), email)]
    email: String,

    #[validate(length(min = 7, max = 20))]
    phone: String,

    file: TempFile,

    #[validate(length(min = 5, max = 500))]
    message: String
}

impl Curriculum {
    fn new(recv_curriculum: RecvCurriculum) -> Curriculum {
        Curriculum {
            name: recv_curriculum.name.0,
            email: recv_curriculum.email.0,
            phone: recv_curriculum.phone.0,
            file: recv_curriculum.file,
            message: recv_curriculum.message.0
        }
    }
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
        
        let mut formfile = form.file;
        let file = FileForm::new(&mut formfile);

        if let Err(err) = file {
            return Err(err);
        }
        let file = file.unwrap();

        let content_type = ContentType::parse(&file.file_type).unwrap();

        let attachment = Attachment::new(formfile.file_name.clone().unwrap()).body(file.file, content_type);

        let body = format!("\nNome: {}\nEmail: {}\nTelefone: {}\n\nMensagem: {}", form.name, form.email, form.phone, form.message);

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


pub async fn send_proposal(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {
    
    if let Err(err) = req.validate() {
        return HttpResponse::BadRequest().json(err);
    } 

    let db = Database::new("proposal", app_state);
    db.insert(req).await
}

pub async fn send_contact_us(app_state: web::Data<AppState>, req: web::Json<User>) -> impl Responder {

    if let Err(err) = req.validate() {
        return HttpResponse::BadRequest().json(err);
    }

    let db = Database::new("contact_us", app_state);
    db.insert(req).await

}

pub async fn send_work_with_us(MultipartForm(form): MultipartForm<RecvCurriculum>) -> impl Responder {
    let form = Curriculum::new(form);

    if let Err(err) = form.validate() {
        return HttpResponse::BadRequest().json(err);
    }

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
    
    if let Err(err) = req.validate() {
        return HttpResponse::BadRequest().json(err);
    }

    let db = Database::new("support", app_state);
    db.insert(req).await
}


