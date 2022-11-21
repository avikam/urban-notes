use crate::todos::{TodoList, todo_list_from_notes};

use sqlx::{Postgres, Row};
use sqlx::{Transaction, Error};

pub async fn get_next<'d>(tx: &mut Transaction<'d, Postgres>, user_id: &str, agent_id: &str, limit: u8) -> Result<(TodoList, bool), Error> {
   let res = read_next(tx, user_id, agent_id, limit).await;

   if let Err(err) = res {
      return Err(err);
   }
   let (last_tid, todo_list) = res.unwrap();

   if last_tid != 0 {
      let res = advance_curser(tx, user_id, agent_id, last_tid).await;
      if let Err(err) = res {
         return Err(err);
      }
   }

   let has_more = todo_list.len() == limit as usize;

   Ok((todo_list, has_more))
}

async fn read_next<'d>(tx: &mut Transaction<'d, Postgres>, user_id: &str, agent_id: &str, limit: u8) -> Result<(i32, TodoList), Error> {
   let res: Result<Vec<(i32, String)>, Error> = sqlx::query(r#"
     SELECT tid, title FROM todo 
     WHERE user_id = $1 AND
     tid > COALESCE((select last_tid from sync_cursor where user_id = $1 and agent_id = $2 limit 1), 1)
     ORDER BY tid ASC limit $3"#
   )
   .bind(user_id).bind(agent_id) .bind(limit as i32)
   .fetch_all(tx)
   .await
   .map(
      |rows|
      rows.into_iter().map(|row| 
         (row.try_get(0).unwrap(), row.try_get(1).unwrap())
    ).collect()
   );

  res.map(|p| p.into_iter().unzip()).map(|(list_of_ids,list_of_todos): (Vec<i32>, Vec<String>)| (
    list_of_ids.iter().max().unwrap_or(&0).to_owned(), 
    todo_list_from_notes(&list_of_todos))
  )
}

async fn advance_curser<'d>(tx: &mut Transaction<'d, Postgres>, user_id: &str, agent_id: &str, last_tid: i32) -> Result<(), Error> {
   sqlx::query(r#"
      INSERT INTO sync_cursor(user_id, agent_id, last_tid) VALUES ($1, $2, $3)
      ON CONFLICT (user_id, agent_id) DO UPDATE SET last_tid = $3 WHERE sync_cursor.user_id = $1 AND sync_cursor.agent_id = $2
   "#)
   .bind(user_id).bind(agent_id) .bind(last_tid)
   .execute(tx)
   .await.map(|_op| ())
}

mod test {
   use super::*;
   use std::{time::Duration, borrow::{Borrow, BorrowMut}};
   use sqlx::{postgres::PgPoolOptions, Pool, Acquire};
   use tokio;

   async fn pool() -> Pool<Postgres> {
       PgPoolOptions::new()
       .max_connections(5)
       .acquire_timeout(Duration::from_secs(3))
       .connect("postgres://postgres:password@localhost/urban_notes").await
       .expect("Can't connect to the database")
   }

   #[tokio::test]
   async fn test_get_next() {
      let p = pool().await;
      let mut tx = p.begin().await.expect("can't initiate tx");

      {
         sqlx::query(r#"INSERT INTO todo(user_id, title, list, idemp_key) VALUES
            ('user1', 'item1', 'list1', 'idemp1'),
            ('user2', 'item2', 'list1', 'idemp2'),
            ('user1', 'item3', 'list1', 'idemp3'),
            ('user1', 'item4', 'list1', 'idemp4'),
            ('user1', 'item5', 'list1', 'idemp5')
            "#
         ).execute(tx.borrow_mut()).await.expect("can't insert test data");
      }

      {
         let mut test_tx = tx.borrow_mut().begin().await.expect("can't start test transaction");

         let (result, _has_more) = get_next(test_tx.borrow_mut(), "user1", "useragent1", 2).await.expect("error getting next");
         assert_eq!(result.len(), 2);
         let r: Vec<&str> = result.into_iter().map(|t| t.name().as_ref()).collect();
         assert_eq!(format!("{:?}", r), r#"["item1", "item3"]"#);
         
         let (result, _has_more) = get_next(test_tx.borrow_mut(), "user1", "useragent1", 2).await.expect("error getting next");
         assert_eq!(result.len(), 2);

         let (result, _has_more) = get_next(test_tx.borrow_mut(), "user1", "useragent1", 2).await.expect("error getting next");
         assert_eq!(result.len(), 0);

         let (result, _has_more) = get_next(test_tx.borrow_mut(), "user1", "useragent1", 2).await.expect("error getting next");
         assert_eq!(result.len(), 0);
      }

      tx.rollback().await.unwrap();
   }

   #[tokio::test]
   async fn test_read_next() {
      let p = pool().await;
      let mut tx = p.begin().await.expect("can't initiate tx");

      {
         sqlx::query(r#"INSERT INTO todo(user_id, title, list, idemp_key) VALUES
            ('user1', 'item1', 'list1', 'idemp1'),
            ('user2', 'item2', 'list1', 'idemp2'),
            ('user1', 'item3', 'list1', 'idemp3'),
            ('user1', 'item4', 'list1', 'idemp4'),
            ('user1', 'item5', 'list1', 'idemp5')
            "#
         ).execute(tx.borrow_mut()).await.expect("can't insert test data");
      }

      {
         let mut test_tx = tx.borrow_mut().begin().await.expect("can't start test transaction");

         let (_cursor, result) = read_next(test_tx.borrow_mut(), "user1", "useragent1", 2).await.expect("error getting next");
         assert_eq!(result.len(), 2);
         let r: Vec<&str> = result.into_iter().map(|t| t.name().as_ref()).collect();
         assert_eq!(format!("{:?}", r), r#"["item1", "item3"]"#);
         
         let (_cursor, result) = read_next(test_tx.borrow_mut(), "user1", "useragent1", 5).await.expect("error getting next");
         assert_eq!(result.len(), 4);

         let (_cursor, result)= read_next(test_tx.borrow_mut(), "user1", "useragent1", 100).await.expect("error getting next");
         assert_eq!(result.len(), 4);
      }


      tx.rollback().await.unwrap();
   }

   async fn get_cursor<'d>(tx: &mut Transaction<'d, Postgres>, user_id: &str, agent_id: &str) -> Option<i32> {
      sqlx::query("SELECT last_tid FROM sync_cursor WHERE user_id = $1 and agent_id = $2")
      .bind(user_id).bind(agent_id)
      .fetch_one(tx.borrow_mut())
      .await.map(|val| val.try_get("last_tid").unwrap()).ok()
   }

   #[tokio::test]
   async fn test_advance_curser() {
      let p = pool().await;
      let mut tx = p.begin().await.expect("can't initiate tx");

      
      {
         assert_eq!(get_cursor(tx.borrow_mut(), "user1", "agent1").await, None);

         advance_curser(tx.borrow_mut(), "user1", "agent1", 10).await.expect("update non existing cursor failed");
         assert_eq!(get_cursor(tx.borrow_mut(), "user1", "agent1").await, Some(10));

         // Should not fail when upserting
         advance_curser(tx.borrow_mut(), "user1", "agent1", 100).await.expect("update existing cursor failed");
         assert_eq!(get_cursor(tx.borrow_mut(), "user1", "agent1").await, Some(100));
      }

      tx.rollback().await.unwrap();
   }

}
