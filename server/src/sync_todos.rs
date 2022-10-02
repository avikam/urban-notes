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

    pub async fn sync_todos(&mut self, todo_list: &TodoList) -> Result<(), String> {
        for item in todo_list {
            if self.sent.contains(&item) {
                continue;
            }

            self.client.post_todo_item(item).await?;
            self.sent.insert(item.to_owned());
        }
        Ok(())
    }
}

pub async fn sync_todos(client: &mut impl PostTodoItem, todo_list: &TodoList) -> Result<(), String> {
    for item in todo_list {
        client.post_todo_item(item).await?;
    }
    Ok(())
}

mod test {
    use super::*;
    use crate::todos::parse_todo_list;
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
    async fn test_sync_todos() -> Result<(), String> {
        let mut client = MockClient::new();
        let todo_list = parse_todo_list("<ul><li>task 1</li><li>task 2</li></ul>");

        sync_todos(&mut client, &todo_list).await
    }

    #[tokio::test]
    async fn test_list_synchronizer() -> Result<(), String> {
        let mut client = MockClient::new();

        {
            let mut syncher = ListSynchronizer::new(&mut client);
            syncher.sync_todos(&parse_todo_list("<ul><li>task 1</li><li>task 2</li></ul>")).await?;
        }
        assert_eq!(client.0.len(), 2);
        
        {
            let mut syncher = ListSynchronizer::new(&mut client);
            syncher.sync_todos(&parse_todo_list("<ul><li>task 1</li><li>task 2</li></ul>")).await?;
        }
        assert_eq!(client.0.len(), 4);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_synchronizer2() -> Result<(), String> {
        struct TestCase { tasks: &'static str, expected: usize };
        let test_cases = vec![
            TestCase { tasks: "<ul><li>task 1</li><li>task 2</li></ul>", expected: 2 },
            TestCase { tasks: "<ul><li>task 1</li><li>task 2</li><li>task 2</li></ul>", expected: 2 },
            TestCase { tasks: "<ul><li>task 1</li><li>task 2</li><li>task 2</li><li>task 1</li></ul>", expected: 2 },
            TestCase { tasks: "<ul><li>task 1</li><li>task 2</li><li>task 2</li><li>task 1</li><li>task 3</li></ul>", expected: 3 },
        ];

        for t in test_cases.iter() {
            let mut client = MockClient::new();
            let mut syncher = ListSynchronizer::new(&mut client);
            syncher.sync_todos(&parse_todo_list(t.tasks)).await?;
            assert_eq!(client.0.len(), t.expected);
        }
        

        Ok(())
    }
}
