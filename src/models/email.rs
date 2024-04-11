use std::{env, io::Read};

use actix_multipart::form::tempfile::TempFile;
use lettre::{message::{header::{self, ContentType}, Attachment, Mailbox, MultiPart, SinglePart}, transport::smtp::authentication::Credentials, Message, SmtpTransport};

use super::curriculum::Curriculum;

struct FileForm {
    file: Vec<u8>,
    file_type: String
}

impl FileForm {
    fn new(temp_file: &mut TempFile) -> Result<FileForm, String> {
        let mut file: Vec<u8> = vec![];
        let _ = temp_file.file.read_to_end(&mut file);

        let file_type: Result<String, String> = match temp_file.content_type.clone().unwrap().to_string() {
            ftype if ftype == String::from("application/pdf") => Ok(String::from("application/pdf")),
            ftype if ftype == String::from("application/msword") => Ok(String::from("application/msword")),
            ftype if ftype == String::from("application/vnd.openxmlformats-officedocument.wordprocessingml.document") => Ok(String::from("application/vnd.openxmlformats-officedocument.wordprocessingml.document")),
            _ => Err(String::from("The file format is invalid."))
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

pub struct Mail {
    host: String,
    port: u16,
    email: String,
    username: String,
    password: String
}

impl Mail {
    pub fn env_init() -> Mail {
        let email = env::var("EMAIL").expect("Enter the EMAIL environment variable correctly");
        let port = env::var("PORT").expect("Enter the PORT environment variable correctly")
            .trim()
            .parse::<u16>()
            .expect("Enter the PORT environment variable correctly");
        let host = env::var("HOST").expect("Enter the HOST environment variable correctly");
        let username = env::var("USERNAME").expect("Enter the USERNAME environment variable correctly");
        let password = env::var("PASSWORD").expect("Enter the PASSWORD environment variable correctly");

        Mail {
            host,
            port,
            email,
            username,
            password
        }
    }

    pub fn new(&self, form: Curriculum) -> Result<(SmtpTransport, Message), String> {
        
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
            .from(self.email.parse::<Mailbox>().unwrap())
            .to(self.email.parse::<Mailbox>().unwrap())
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
            .port(self.port)
            .build();

        Ok((mailer, message))
    }
}
