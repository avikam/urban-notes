use crate::anydo;
use crate::anydo::AnydoClient;
use crate::todos::TodoList;

pub async fn sync_todos(client: &AnydoClient, todo_list: &TodoList) -> Result<(), &'static str> {
    for todo in todo_list {
        client
            .post_task(
                anydo::Task::new()
                    .set_title(todo.name())
                    .set_category("0lJw090p7r3WG5OZ37X1p30k")
                    .add_label("L09XEs7V5FWpQJiN4aW0TWcG"),
            )
            .await
            .map_err(|e| {
                println!("error! {:?}", e);
                "error!"
            })?;
    }
    Ok(())
}

mod test {
    use super::*;
    use crate::todos::parse_todo_list;
    use std::env;
    use tokio;

    #[tokio::test]
    async fn test_sync_todos() -> Result<(), &'static str> {
        let token = env::var("ANYDO_TOKEN").unwrap();
        let client = AnydoClient::new(token.as_ref());
        let todo_list = parse_todo_list("<ul><li>task 1</li><li>task 2</li></ul>");

        sync_todos(&client, &todo_list).await
    }
}
