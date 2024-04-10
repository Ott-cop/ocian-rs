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
    username: String,
    password: String,
    from_email: String,
    to_email: String
}

impl Mail {
    pub fn env_init() -> Mail {
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
