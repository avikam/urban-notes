use std::default::Default;
use std::slice::Iter;
use regex::RegexBuilder;

pub struct Todo {
    name: String,
    completed: bool,
}

impl Default for Todo {
    fn default() -> Self {
        Todo {
            name: "".to_string(),
            completed: false,
        }
    }
}

impl Todo {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

pub struct TodoList {
    todo_list: Vec<Todo>,
}

impl Default for TodoList {
    fn default() -> Self {
        TodoList {
            todo_list: Default::default(),
        }
    }
}

pub fn parse_todo_list(text: &str) -> TodoList {
    let r = RegexBuilder::new(r"<li>\s*(?P<description>.*?)\s*</li>")
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    let todo_list: Vec<Todo> = r
        .captures_iter(text)
        .map(|c| Todo {
            name: c["description"].to_string(),
            ..Default::default()
        })
        .collect();
    TodoList { todo_list }
}

impl TodoList {
    pub fn len(&self) -> usize {
        self.todo_list.len()
    }
}

impl<'a> IntoIterator for &'a TodoList {
    type Item = <&'a Vec<Todo> as IntoIterator>::Item;
    type IntoIter = <&'a Vec<Todo> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter { (&self.todo_list).into_iter() }
}

mod test {
    use super::parse_todo_list;

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

        let joined = l1.into_iter().map(|t| t.name.as_ref()).collect::<Vec<&str>>().join(", ");
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
