use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use std::default::Default;

lazy_static! {
    static ref TASK_REGEX: Regex = RegexBuilder::new(r"<li>\s*(?P<description>.*?)\s*</li>")
        .dot_matches_new_line(true)
        .build()
        .expect("TASK_REGEX error");
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct TodoItem {
    name: String,
    completed: bool,
}

impl Default for TodoItem {
    fn default() -> Self {
        TodoItem {
            name: "".to_string(),
            completed: false,
        }
    }
}

impl TodoItem {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl<'a> AsRef<str> for TodoItem {
    fn as_ref(&self) -> &str { 
        self.name.as_ref()
    }
}

pub struct TodoList {
    todo_list: Vec<TodoItem>,
}

impl Default for TodoList {
    fn default() -> Self {
        TodoList {
            todo_list: Default::default(),
        }
    }
}

pub fn parse_todo_list(text: &str) -> TodoList {
    let todo_list: Vec<TodoItem> = TASK_REGEX
        .captures_iter(text)
        .map(|c| TodoItem {
            name: c["description"].to_string(),
            ..Default::default()
        })
        .collect();
    TodoList { todo_list }
}

pub fn todo_list_from_notes(text: &[&str] ) -> TodoList {
    let todo_list = text.iter().map(|t| TodoItem {completed: false, name: (*t).to_owned()}).collect();
    TodoList { todo_list }
}

impl TodoList {
    pub fn len(&self) -> usize {
        self.todo_list.len()
    }
}

impl<'a> IntoIterator for &'a TodoList {
    type Item = <&'a Vec<TodoItem> as IntoIterator>::Item;
    type IntoIter = <&'a Vec<TodoItem> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&self.todo_list).into_iter()
    }
}

mod test {
    use super::*;

    #[test]
    fn test_task_as_ref() {
        let t = TodoItem { name: "hello".to_string(), ..Default::default() };
        assert_eq!("hello", t.as_ref() as &str);
    }

    #[test]
    fn test_parse_todo_list() -> Result<(), String> {
        let result = parse_todo_list("<ul>\n<li>The good place </li>\n<li>Parasite </li>\n<li>Involve me more in project Metal (today is 5/5/2022)</li>\n</ul>\nwords words \n<ul><li>task 1</li><li>task 2</li></ul>");
        assert_eq!(result.todo_list.len(), 5);

        Ok(())
    }

    #[test]
    fn test_into_iterator() {
        let l1 = parse_todo_list("<ul><li>task 1</li><li>task 2</li></ul>");

        let r: Vec<&str> = l1.into_iter().map(|t| t.name.as_ref()).collect();
        assert_eq!(format!("{:?}", r), r#"["task 1", "task 2"]"#);

        let joined = l1
            .into_iter()
            .map(|t| t.name.as_ref())
            .collect::<Vec<&str>>()
            .join(", ");
        assert_eq!("task 1, task 2", joined);
        let tasks = vec!["task 1".to_string(), "task 2".to_string()];

        let mut iter = tasks.iter();
        for task in &l1 {
            assert_eq!(Some(&task.name), iter.next());
        }

        let mut iter = tasks.iter();
        for task in &l1 {
            assert_eq!(task.name, *iter.next().unwrap());
        }

        let mut iter = tasks.iter();
        let mut i = l1.into_iter();
        while let Some(task) = i.next() {
            assert_eq!(Some(&task.name), iter.next());
        }
        let mut iter = tasks.iter();
        let mut j = (&l1).into_iter();
        while let Some(task) = j.next() {
            assert_eq!(task.name, *iter.next().unwrap());
        }
    }
}
