#[macro_use]
extern crate rocket;

mod anydo;
mod sync_todos;
mod todos;
mod storage;

use std::cmp;
use std::io::Cursor;
use std::time::Duration;

use rocket::serde::json::Json;
use rocket::State;
use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::ContentType;
use rocket::http::Status;

use serde::Deserialize;
use std::borrow::{Cow, BorrowMut};
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;
use tokio::signal;

use sqlx::postgres::{Postgres, PgPoolOptions};
use sqlx::Pool;

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

struct PostTodosErr {
    http_status: Status,
    err: String
}

impl<'a> Responder<'a, 'a> for PostTodosErr {
    fn respond_to(self, _: &Request) -> response::Result<'a> {
        Response::build()
            .header(ContentType::Plain)
            .sized_body(self.err.len(), Cursor::new(self.err))
            .status(self.http_status)
            .ok()
    }
}

#[post("/push?<user_id>", data = "<notes>")]
async fn push(pool: &State<Pool<Postgres>>, queue: &State<TodoQueue>, notes: Json<Vec<Note<'_>>>, user_id: Option<String>) -> Result<Json<String>, PostTodosErr> {
    if user_id.is_none() {
        return Err(PostTodosErr{ http_status: Status::BadRequest, err: "Must provide user_id".to_string() })
    }
    let user_id = user_id.unwrap();

    let x: Vec<&str> = notes.iter().map_while(|n| 
        match n.todo.as_ref().is_empty() {
            true => None,
            false => Some(n.todo.as_ref())
        }
    ).collect();

    if x.len() != notes.len() {
        return Err(PostTodosErr{ http_status: Status::BadRequest, err: "Found empty todo".to_string() })
    }
    
    let mut tx = pool
                                        .begin()
                                        .await
                                        .map_err(|_err| 
                                            PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() }
                                        )?;

    let todo_list = todos::todo_list_from_notes(&x);

    let store_result = storage::todos::store_todos(&mut tx, user_id.as_str(), &todo_list)
                                        .await
                                        .map_err(|_err| 
                                            PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() }
                                        )?;
    dbg!(store_result);
    
    tx.commit().await.map_err(|_err| PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() })?;

    let sender = queue.sender.clone();

    // TODO: Expose error
    let spawn_result = spawn_blocking(move || {
        sender.blocking_send(todo_list)
    }).await.unwrap();

    if let Err(send_error) = spawn_result {
        println!("SendError: {}", send_error);
        panic!()
    }

    Ok(Json(
        "OK".to_string()
    ))
}

#[get("/pull?<user_id>&<agent_id>&<limit>")]
async fn pull(pool: &State<Pool<Postgres>>, user_id: Option<String>, agent_id: Option<String>, limit: Option<i32>) -> Result<Json<Vec<String>>, PostTodosErr> {
    if user_id.is_none() || agent_id.is_none() {
        return Err(PostTodosErr{ http_status: Status::BadRequest, err: "Must provide user_id and agent_id".to_string() })
    }

    let user_id = user_id.unwrap();
    let agent_id = agent_id.unwrap();
    let limit = cmp::min(limit.unwrap_or(5), 10);

    let mut tx = pool
        .begin()
        .await
        .map_err(|_err| 
            PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() }
        )?;

    let res = storage::cursors::get_next(tx.borrow_mut(), user_id.as_str(), agent_id.as_str(), limit).await.map_err(|_err| PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() })?;

    tx.commit().await.map_err(|_err| PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() })?;

    Ok(Json(
        res.into_iter().map(|t| t.name().to_string()).collect()
    ))
}

#[rocket::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect("postgres://postgres:password@localhost/urban_notes").await
        .expect("Can't connect to the database");
    

    let (sender, mut receiver) = mpsc::channel::<todos::TodoList>(10);

    let token = std::env::var("ANYDO_TOKEN").expect("no anydo token");
    let handle = tokio::spawn(async move {
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
    .manage(pool)
    .mount("/", routes![push, pull])
    .launch();

    result.await.expect("server failed unexpectedly");
    handle.await.expect("recv failed unexpectedly");
}
