#[macro_use] extern crate rocket;

mod todos;

use serde::{Deserialize};
use rocket::serde::json::Json;
use rocket::serde::json::serde_json;
use rocket::form::Form;
use rocket::Data;
use rocket::response::Debug;
use std::borrow::Cow;

#[derive(Deserialize)]
struct Note<'r> {
    folder: &'r str,
    name: &'r str,

    #[serde(borrow)]
    text: Cow<'r, str>,

    #[serde(borrow)]
    body: Cow<'r,str> ,
}

#[post("/notes", data = "<note>")]
fn index(note: Json<Note<'_>>) -> String {
    let todo_list = todos::parse_todo_list(&note.body);
    todo_list.into_iter().map(|t| t.name()).collect::<Vec<&str>>().join(", ")
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
