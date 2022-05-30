use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use base64;
use reqwest::{header, Client, ClientBuilder, Error, Method, Request, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

static TASKS_URL: &str = "https://sm-prod2.any.do/me/tasks";

#[derive(Debug)]
pub enum AnydoError {
    ReqwestError(Error),
    ClientError(String),
}

pub struct AnydoClient {
    client: Client,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    id: String,
    title: String,
    category_id: String,
    labels: Option<HashSet<String>>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

pub struct TaskListResult {
    result_list: Vec<Rc<Task>>,

    id_map: RefCell<HashMap<String, Rc<Task>>>,
}

fn new_anydo_id() -> String {
    let id = Uuid::new_v4();
    let bytes = id.as_bytes();
    base64::encode_config(bytes, base64::URL_SAFE)
}

impl Default for Task {
    fn default() -> Self {
        Task {
            id: new_anydo_id(),
            title: Default::default(),
            category_id: Default::default(),
            labels: Default::default(),
            extra: Default::default(),
        }
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.title == other.title && self.extra == other.extra
    }
}

impl Task {
    pub fn new() -> Self {
        Default::default()
    }

    fn from_task(task: &Task) -> Self {
        let mut t = task.clone();
        t.id = new_anydo_id();
        t
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_owned();
        self
    }

    pub fn set_category(&mut self, category: &str) -> &mut Self {
        self.category_id = category.to_owned();
        self
    }


    pub fn add_label(&mut self, label: &str) -> &mut Self {
        match &mut self.labels {
            None => {
                let mut labels = HashSet::new();
                labels.insert(label.to_owned());
                self.labels = Some(labels);
            }
            Some(labels) => {
                labels.insert(label.to_owned());
            }
        }
        self
    }
}

impl TaskListResult {
    fn from_list(list: &mut Vec<Task>) -> TaskListResult {
        TaskListResult {
            result_list: list
                .into_iter()
                .map(|t| Rc::new(std::mem::take(t)))
                .collect(),
            id_map: Default::default(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Rc<Task>> {
        self.result_list.iter()
    }

    pub fn by_id(&self, id: &str) -> Option<Rc<Task>> {
        if self.result_list.len() == 0 {
            return None;
        }

        if self.id_map.borrow().len() == 0 {
            self.init_id_map();
        }

        self.id_map.borrow().get(id).map(|t| t.clone())
    }

    fn init_id_map(&self) {
        self.id_map.replace(
            self.result_list
                .iter()
                .map(|t| (t.id.clone(), t.clone()))
                .collect(),
        );
    }
}

impl AnydoClient {
    pub fn new(login_token: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::HeaderName::from_lowercase(b"x-anydo-auth").unwrap(),
            login_token.parse().unwrap(),
        );

        let client_builder = ClientBuilder::new().default_headers(headers);

        AnydoClient {
            client: client_builder.build().unwrap(),
        }
    }

    pub async fn list_tasks(
        &self,
        include_done: bool,
        include_deleted: bool,
    ) -> Result<TaskListResult, AnydoError> {
        let url = Url::parse_with_params(
            TASKS_URL,
            &[("includeDone", "false"), ("includeDeleted", "false")],
        )
        .unwrap();

        let mut res = self
            .client
            .execute(Request::new(Method::GET, url))
            .await
            .map_err(|e| AnydoError::ReqwestError(e))?
            .json::<Vec<Task>>()
            .await
            .map_err(|e| AnydoError::ReqwestError(e))?;
        Ok(TaskListResult::from_list(&mut res))
    }

    pub async fn post_task(&self, task: &Task) -> Result<Vec<Task>, AnydoError> {
        let params = vec![task];
        let response = self
            .client
            .post(TASKS_URL)
            .json(&params)
            .send()
            .await
            .map_err(|e| AnydoError::ReqwestError(e))?;

        if response.status() != 200 {
            return Err(AnydoError::ClientError(format!(
                "Status: {}",
                response.status()
            )));
        }

        response
            .json::<Vec<Task>>()
            .await
            .map_err(|e| AnydoError::ReqwestError(e))
    }
}

mod test {
    use super::*;
    use std::env;
    use tokio;

    fn c() -> AnydoClient {
        let token = env::var("ANYDO_TOKEN").unwrap();
        AnydoClient::new(token.as_ref())
    }

    #[tokio::test]
    async fn test_list_tasks() -> Result<(), AnydoError> {
        let client = c();
        let anydo_tasks = client.list_tasks(false, false).await?;
        println!("{:?}\n", anydo_tasks.result_list);

        let titles: Vec<&str> = anydo_tasks.iter().map(|t| t.title.as_ref()).collect();
        println!("{}\n", titles.join("\n"));
        
        let lables: Vec<String> = (&anydo_tasks)
            .iter()
            .filter(|t| t.labels.is_some())
            .map(|t| format!("{:?}", t.labels.as_ref().unwrap()))
            .collect();
        println!("lables = {}", lables.join("\n"));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_tasks() -> Result<(), AnydoError> {
        let client = c();
        let anydo_tasks = client.list_tasks(false, false).await?;
        let id = anydo_tasks.result_list[3].id.as_ref();
        assert_eq!(
            anydo_tasks.by_id(id),
            Some(anydo_tasks.result_list[3].clone())
        );
        assert_eq!(
            anydo_tasks.by_id(id),
            Some(anydo_tasks.result_list[3].clone())
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_post_tasks() -> Result<(), AnydoError> {
        let client = c();
        let mut task = Task::new();
        task.set_title("Test Task!")
            .set_category("0lJw090p7r3WG5OZ37X1p30k")
            .add_label("A_BeAbayusmqiKzjlff9xw==")
            .add_label("L09XEs7V5FWpQJiN4aW0TWcG")
            .add_label("A_BeAbayusmqiKzjlff9xw==");

        let res = client.post_task(&task).await?;
        println!("task = {:?}", res);

        Ok(())
    }
}
