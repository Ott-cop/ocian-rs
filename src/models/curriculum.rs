use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use validator::Validate;

#[derive(MultipartForm)]
pub struct RecvCurriculum {
    name: Text<String>,
    email: Text<String>,
    phone: Text<String>,
    file: TempFile,
    message: Text<String>
}

#[derive(Validate)]
pub struct Curriculum {
    #[validate(length(min = 5, max = 50))]
    pub name: String,

    #[validate(length(min = 7, max = 50), email)]
    pub email: String,

    #[validate(length(min = 7, max = 20))]
    pub phone: String,

    pub file: TempFile,

    #[validate(length(min = 5, max = 500))]
    pub message: String
}

impl Curriculum {
    pub fn new(recv_curriculum: RecvCurriculum) -> Curriculum {
        Curriculum {
            name: recv_curriculum.name.0,
            email: recv_curriculum.email.0,
            phone: recv_curriculum.phone.0,
            file: recv_curriculum.file,
            message: recv_curriculum.message.0
        }
    }
}