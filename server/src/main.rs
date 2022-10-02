#[macro_use]
extern crate rocket;

mod anydo;
mod sync_todos;
mod todos;


use rocket::form::Form;
use rocket::response::Debug;
use rocket::serde::json::serde_json;
use rocket::serde::json::Json;
use rocket::{State, Data, Shutdown};
use rocket::tokio::task::spawn_blocking;
use rocket::tokio;

use serde::Deserialize;
use std::borrow::Cow;
use std::thread;
use std::sync::mpsc;

#[derive(Deserialize)]
struct Note<'r> {
    folder: &'r str,
    name: &'r str,

    #[serde(borrow)]
    text: Cow<'r, str>,

    #[serde(borrow)]
    body: Cow<'r, str>,
}

struct TodoQueue {
    sender: mpsc::SyncSender<todos::TodoList>
}

#[post("/notes", data = "<note>")]
async fn notes(queue: &State<TodoQueue>, note: Json<Note<'_>>) -> String {
    let todo_list = todos::parse_todo_list(&note.body);
    let sender = queue.sender.clone();

    // TODO: Expose error
    spawn_blocking(move || sender.send(todo_list) ).await;
    
    "OK".to_string()
}

#[rocket::main]
async fn main() {
    let (sender, receiver) = mpsc::sync_channel::<todos::TodoList>(10);

    let handle = thread::spawn(move || {
        let token = std::env::var("ANYDO_TOKEN").unwrap();
        let mut client = anydo::AnydoClient::new(token.as_ref());

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        for todo_list in receiver.recv() {
            println!("recived: {}", todo_list.len());
            
            let res = rt.block_on(
                sync_todos::sync_todos(&mut client, &todo_list)
            );
            
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
                .join(", ");
        }
    });

    let result = rocket::build()
    .manage(TodoQueue{sender: sender})
    .mount("/", routes![notes])
    .launch()
    .await;

    result.expect("server failed unexpectedly");

    handle.join().unwrap();
}
