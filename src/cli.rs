use crate::core::orth;
use std::error::Error;
use std::io;

fn execute_int(string: &str) {
    let interpreted = orth::interpret(string);
    println!("{}", interpreted);
}

pub fn mainloop() -> Result<(), Box<dyn Error>> {
    loop {
        eprint!("> ");
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let mut iter = buf.trim().split_whitespace();
        match iter.next().unwrap_or("") {
            "exit" | ";" => break Ok(()),
            "int" => execute_int(iter.next().unwrap_or("")),
            "" => (),
            other => println!("Unknown command: {}", other)
        }
    }
}