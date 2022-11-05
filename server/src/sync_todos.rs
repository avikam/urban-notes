use crate::anydo;
use crate::anydo::AnydoClient;
use crate::todos::{TodoItem, TodoList};
use std::collections::HashSet;
use async_trait::async_trait;

#[async_trait]
pub trait PostTodoItem {
    async fn post_todo_item(&mut self, item: &TodoItem) -> Result<(), String>;
}

#[async_trait]
impl PostTodoItem for AnydoClient {
    async fn post_todo_item(&mut self, item: &TodoItem) -> Result<(), String> {
        self
        .post_task(
                anydo::Task::new()
                    .set_title(item.name())
                    .set_category("0lJw090p7r3WG5OZ37X1p30k")
                    .add_label("L09XEs7V5FWpQJiN4aW0TWcG"),
            )
            .await
            .map_err(|e| {
                println!("error! {:?}", e);
                "error!".to_string()
            }).map(|_| (()))
    }
}

pub struct ListSynchronizer<'a, T: PostTodoItem> {
    client: &'a mut T,

    // send log
    sent: HashSet<TodoItem>,
}

impl<'a, T: PostTodoItem> ListSynchronizer<'a, T> {
    pub fn new(client: &'a mut T) -> Self {
        ListSynchronizer {
            client: client,
            sent: HashSet::new(),
        }
    }

    pub async fn sync_todos(&mut self, todo_list: &TodoList) -> Result<usize, String> {
        let mut i = 0;
        for item in todo_list {
            if true || self.sent.contains(&item) {
                continue;
            }

            self.client.post_todo_item(item).await?;
            self.sent.insert(item.to_owned());
            i += 1;
        }
        Ok(i)
    }
}

mod test {
    use super::*;
    use crate::todos::todo_list_from_notes;
    use std::env;
    use tokio;

    struct MockClient(Vec<TodoItem>);
    
    impl MockClient {
        fn new() -> Self { Self(vec![]) }
     }

    #[async_trait]
    impl PostTodoItem for MockClient {
        async fn post_todo_item(&mut self, item: &TodoItem) -> Result<(), String> {
            self.0.push(item.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_list_synchronizer() -> Result<(), String> {
        let mut client = MockClient::new();
        let todos = vec!["task 1", "task 2"];
        let tasks = todo_list_from_notes(&todos);

        {
            let mut syncher = ListSynchronizer::new(&mut client);
            syncher.sync_todos(&tasks).await?;
        }
        assert_eq!(client.0.len(), 2);
        
        {
            let mut syncher = ListSynchronizer::new(&mut client);
            syncher.sync_todos(&tasks).await?;
        }
        assert_eq!(client.0.len(), 4);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_synchronizer2() -> Result<(), String> {
        struct TestCase { tasks: Vec<&'static str>, expected: usize }
        let test_cases = vec![
            TestCase { tasks: vec!["task 1", "task 2"], expected: 2 },
            TestCase { tasks: vec!["task 1", "task 2", "task 2"], expected: 2 },
            TestCase { tasks: vec!["task 1", "task 2", "task 2", "task 1"], expected: 2 },
            TestCase { tasks: vec!["task 1", "task 2", "task 2", "task 1", "task 3"], expected: 3 },
        ];

        for t in test_cases.iter() {
            let mut client = MockClient::new();
            let mut syncher = ListSynchronizer::new(&mut client);
            syncher.sync_todos(&&todo_list_from_notes(&t.tasks)).await?;
            assert_eq!(client.0.len(), t.expected);
        }
        

        Ok(())
    }
}
