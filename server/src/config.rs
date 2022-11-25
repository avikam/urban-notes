use rocket::serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    database_hostname: String,
    database_username: String,
    database_password: String
}

impl Config {
    pub fn database_connection_string(&self) -> String {
        format!("postgres://{}:{}@{}/urban_notes", self.database_username, self.database_password, self.database_hostname)
    }
}
