#[macro_use] extern crate rocket;

use serde::{Deserialize};
use rocket::serde::json::Json;
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
    format!("{} / {}", note.folder, note.name)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
