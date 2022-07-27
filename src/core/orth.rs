use std::collections::HashMap;
use regex::{Regex, Captures};
use std::fs;

fn init_orth() -> HashMap<String, String> {
    let contents = fs::read_to_string("src/core/orth/command.txt").unwrap();
    let mut commands = HashMap::new();
    for line in contents.lines() {
        let pair: Vec<&str> = line.trim().split_whitespace().collect();
        commands.insert(pair[1], pair[0]);
    }
    let mut map = HashMap::new();
    for (name, cmd) in commands.into_iter() {
        let contents = fs::read_to_string(format!("src/core/orth/{}.txt", name)).unwrap();
        for line in contents.lines() {
            let pair: Vec<&str> = line.trim().split_whitespace().collect();
            map.insert(format!("{}{}", cmd, pair[0]), String::from(pair[1]));
        }
    }
    let contents = fs::read_to_string("src/core/orth/combination.txt").unwrap();
    for line in contents.lines() {
        let pair: Vec<&str> = line.trim().split_whitespace().collect();
        map.insert(String::from(pair[0]), String::from(pair[1]));
    }
    map
}

pub fn interpret(string: &str) -> String {
    lazy_static! {
        static ref ORTH: HashMap<String, String> = init_orth();
        static ref COMMAND: Regex = Regex::new(r"\\(.{2})").unwrap();
        static ref COMBINATION: Regex = Regex::new(r"\{(\w+)\}").unwrap();
    }
    let repl_closure = |caps: &Captures| {
        match ORTH.get(&caps[1]) {
            Some(repl) => repl.clone(),
            None => String::from(&caps[1])
        }
    };
    let string = COMBINATION.replace_all(string, repl_closure);
    let string = COMMAND.replace_all(&string, repl_closure);
    string.into_owned()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_index() {
        let string = r"se\~n{o}rita";
        assert_eq!(interpret(string), String::from("señørita"));
    }
}