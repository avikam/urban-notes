#[macro_use]
extern crate rocket;

mod anydo;
mod sync_todos;
mod todos;

use rocket::serde::json::Json;
use rocket::State;

use serde::Deserialize;
use std::borrow::Cow;
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;
use tokio::signal;

#[derive(Deserialize)]
struct Note<'r> {
    folder: &'r str,
    name: &'r str,

    #[serde(borrow)]
    todo: Cow<'r, str>,
}

struct TodoQueue {
    sender: mpsc::Sender<todos::TodoList>
}

#[post("/notes", data = "<notes>")]
async fn notes(queue: &State<TodoQueue>, notes: Json<Vec<Note<'_>>>) -> String {
    let x: Vec<&str> = notes.iter().map(|n| n.todo.as_ref()).collect();
    let todo_list = todos::todo_list_from_notes(&x);
    
    let sender = queue.sender.clone();

    // TODO: Expose error
    let spawn_result = spawn_blocking(move || {
        sender.blocking_send(todo_list)
    }).await.unwrap();

    if let Err(send_error) = spawn_result {
        println!("SendError: {}", send_error);
        panic!()
    }

    "OK".to_string()
}

#[rocket::main]
async fn main() {
    let (sender, mut receiver) = mpsc::channel::<todos::TodoList>(10);

    let handle = tokio::spawn(async move {
        println!("started recv loop");
        let token = std::env::var("ANYDO_TOKEN").unwrap();
        let mut client = anydo::AnydoClient::new(token.as_ref());

        let mut ls = sync_todos::ListSynchronizer::new(&mut client);

        loop {
            tokio::select! {
                Some(todo_list) = receiver.recv() =>  {
                println!("recived: {}", todo_list.len());
                
                let res = ls.sync_todos(&todo_list).await;
                match res {
                    Err(s) => {
                        println!("Error!: {}", s);
                    }
                    Ok(j) => {
                        println!("success! inserted {} items", j);
                    }
                }
                
                todo_list
                    .into_iter()
                    .map(|t| t.name())
                    .collect::<Vec<&str>>()
                    .join(", ");
                },

                _ = signal::ctrl_c() => {
                    break;
                }
            }
        }
        
        println!("recv closed");
    });

    let result = rocket::build()
    .manage(TodoQueue{sender: sender})
    .mount("/", routes![notes])
    .launch();

    result.await.expect("server failed unexpectedly");
    handle.await.expect("recv failed unexpectedly");
}
