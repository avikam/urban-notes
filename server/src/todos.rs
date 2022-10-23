use std::default::Default;

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
    fn test_into_iterator() {
        let todos = vec!("task 1", "task 2");
        let l1 = todo_list_from_notes(&todos);

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
