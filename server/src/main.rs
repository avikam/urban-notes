#[macro_use]
extern crate rocket;

mod config;
mod token;
mod anydo;
mod sync_todos;
mod todos;
mod storage;

use std::cmp;
use std::io::Cursor;
use std::time::Duration;

use rocket::fairing::AdHoc;
use rocket::serde::json::Json;
use rocket::State;
use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::ContentType;
use rocket::http::Status;

use serde::{Deserialize, Serialize};
use std::borrow::{Cow, BorrowMut};
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;
use tokio::signal;

use sqlx::postgres::{Postgres, PgPoolOptions};
use sqlx::Pool;

use config::Config;

use token::Token;
use todos::{TodoList, TodoItem};

#[derive(Deserialize)]
struct Note<'r> {
    folder: &'r str,

    name: &'r str,

    #[serde(borrow)]
    todo: Cow<'r, str>,
}

#[derive(Serialize)]
struct PullResponse {
    todos: Vec<Reminder>,
    has_more: bool
}

#[derive(Serialize)]
struct Reminder {
    name: String,
    list: String
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

#[post("/push", data = "<notes>")]
async fn push(
    token: Token,
    pool: &State<Pool<Postgres>>, 
    queue: &State<TodoQueue>, 
    notes: Json<Vec<Note<'_>>>, 
) -> Result<Json<String>, PostTodosErr> {

    let todo_list = TodoList{
        todo_list: notes.iter().map_while(|n| 
            match n.todo.as_ref().is_empty() {
                true => None,
                false => Some(TodoItem::default().set_name(n.todo.as_ref()).set_list_name(n.folder).to_owned())
            }
        ).collect()
    };

    if todo_list.len() != notes.len() {
        return Err(PostTodosErr{ http_status: Status::BadRequest, err: "Found empty todo".to_string() })
    }
    
    let mut tx = pool
                                        .begin()
                                        .await
                                        .map_err(|_err| 
                                            PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() }
                                        )?;

    let _store_result = storage::todos::store_todos(&mut tx, token.user().as_str(), &todo_list)
                                        .await
                                        .map_err(|_err| 
                                            PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() }
                                        )?;
    
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

#[get("/pull?<agent_id>&<limit>")]
async fn pull(
    token: Token,
    pool: &State<Pool<Postgres>>, 
    agent_id: Option<String>, 
    limit: Option<u8>
) -> Result<Json<PullResponse>, PostTodosErr> {
    let agent_id = agent_id.unwrap();
    let limit = cmp::min(limit.unwrap_or(5), 100);

    let mut tx = pool
        .begin()
        .await
        .map_err(|_err| 
            PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() }
        )?;

    let (res, has_more) = storage::cursors::get_next(
        tx.borrow_mut(), token.user().as_str(), agent_id.as_str(), limit
    ).await.map_err(|_err| PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() })?;

    tx.commit().await.map_err(|_err| PostTodosErr{ http_status: Status::InternalServerError, err: "DB Error".to_string() })?;

    Ok(Json(
        PullResponse {
            has_more:  has_more,
            todos: res.into_iter().map(|t| Reminder { name: t.name().to_string(), list: t.list_name().to_string() }).collect()
        }
    ))
}

#[rocket::main]
async fn main() {
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
    .attach(AdHoc::on_ignite("Database Config", |rocket| async {
        let config: Config = rocket.figment().extract().expect("Failed Reading Config!");

        let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(config.database_connection_string().as_str()).await
        .expect("Can't connect to the database");

        rocket.manage(pool)
    }))
    .mount("/", routes![push, pull])
    .launch();

    result.await.expect("server failed unexpectedly");
    handle.await.expect("recv failed unexpectedly");
}
