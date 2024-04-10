use validator::Validate;

#[derive(serde::Deserialize, serde::Serialize, Validate)]
pub struct User {
    #[validate(length(min = 5, max = 50))]
    pub name: String,

    #[validate(length(min = 7, max = 50), email)]
    pub email: String,

    #[validate(length(min = 7, max = 20))]
    pub phone: String,

    #[validate(length(min = 5, max = 50))]
    pub subject: String,

    #[validate(length(min = 5, max = 500))]
    pub message: String
}