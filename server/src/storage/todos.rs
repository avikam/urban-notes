use crate::todos::{TodoList};

use sqlx::Postgres;
// use sqlx::postgres::Postgres;
use sqlx::{Transaction, Error, Row};

use sqlx::types::chrono::NaiveDateTime;

fn prefix(s: &str, k: usize) -> &str {
    let idx = s.char_indices().nth(k).map(|(idx, _)| idx).unwrap_or(s.len());
    &s[0..idx]
}

pub async fn store_todos<'d>(tx: &mut Transaction<'d, Postgres>, user_id: &str, todo_list: &TodoList) -> Result<NaiveDateTime, Error> {
    // pivot
    let mut titles = Vec::new();
    let mut lists = Vec::new();
    let mut idemps = Vec::new();

    let sorted = todo_list.sorted();
    for item in sorted.into_iter() {
        titles.push(item.name());

        lists.push(item.list_name());

        idemps.push(
            format!("{:x}", item.idmep())
        );
    }

    sqlx::query(
        r#"WITH 
        request AS (
            SELECT title, list, idemp_key
            FROM UNNEST($1,$2,$3) AS a(title, list, idemp_key)
        ) 
        INSERT INTO todo (user_id, title, list, idemp_key)
        SELECT $4 user_id, title, list, idemp_key FROM request
        ON CONFLICT DO NOTHING
        "#,
    ).bind(&titles).bind(&lists).bind(&idemps).bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| dbg!(e) )?;

    sqlx::query(
        r#"WITH 
        request AS (
            SELECT idemp_key
            FROM UNNEST($1) AS a(idemp_key)
        ),
        records AS (
            SELECT request.idemp_key, todo.tid, todo.added FROM request INNER JOIN todo ON todo.idemp_key = request.idemp_key
        )
        SELECT max(added) FROM records;
        "#,
    ).bind(&idemps)
    .fetch_one(tx)
    .await
    .map(|row| row.try_get(0).unwrap())
    .map_err(|e| dbg!(e) )
}



mod test {
    use super::*;
    use crate::todos::todo_list_from_notes;
    use std::{env, time::Duration, borrow::{Borrow, BorrowMut}};
    use sqlx::{postgres::PgPoolOptions, Pool};
    use tokio;

    async fn pool() -> Pool<Postgres> {
        PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect("postgres://postgres:password@localhost/urban_notes").await
        .expect("Can't connect to the database")
    }

    async fn count_todos<'d>(tx: &mut Transaction<'d, Postgres>) -> i64 {
        sqlx::query(
            "select count(*) from todo;"
        ).fetch_one(&mut *tx).await.map(|row| row.try_get(0).unwrap()).expect("can't count todos")
    }

    #[tokio::test]
    async fn test_add_todos() {
        let todos = vec!["task 2", "task 1"];
        let todo_list = todo_list_from_notes(&todos);

        let p = pool().await;
        let mut tx = p.begin().await.expect("can't initiate tx");
        
        let c = count_todos(&mut tx).await;
        assert!(c == 0);


        store_todos(&mut tx, "user", &todo_list).await.unwrap();
        let c = count_todos(&mut tx).await;
        assert!(c == 2);

        // re-add the same todos does nothing 
        let todos = vec!["task 1", "task 2"];
        let todo_list = todo_list_from_notes(&todos);
        
        store_todos(&mut tx, "user", &todo_list).await.unwrap();
        let c = count_todos(&mut tx).await;
        assert!(c == 2);

        tx.rollback();
    }
}

