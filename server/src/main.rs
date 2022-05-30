#[macro_use]
extern crate rocket;

mod anydo;
mod sync_todos;
mod todos;

use rocket::form::Form;
use rocket::response::Debug;
use rocket::serde::json::serde_json;
use rocket::serde::json::Json;
use rocket::Data;
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Deserialize)]
struct Note<'r> {
    folder: &'r str,
    name: &'r str,

    #[serde(borrow)]
    text: Cow<'r, str>,

    #[serde(borrow)]
    body: Cow<'r, str>,
}

#[post("/notes", data = "<note>")]
async fn notes(note: Json<Note<'_>>) -> String {
    let token = std::env::var("ANYDO_TOKEN").unwrap();
    let client = anydo::AnydoClient::new(token.as_ref());

    let todo_list = todos::parse_todo_list(&note.body);
    let res = sync_todos::sync_todos(&client, &todo_list).await;
    match res {
        Err(s) => {
            println!("Error!: {}", s);
        }
        Ok(_) => {
            println!("success!");
        }
    }
    todo_list
        .into_iter()
        .map(|t| t.name())
        .collect::<Vec<&str>>()
        .join(", ")
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![notes])
}
