use crate::core::Babel;
use crate::core::Valid;
use crate::core::orth;
use crate::core::pos::PoS;
use std::error::Error;
use std::io;

pub struct Cli {
    babel: Babel
}

impl Cli {
    pub fn new() -> Cli {
        Cli { babel: Babel::new() }
    }

    fn fetch(prompt: &str) -> io::Result<String> {
        eprint!("{} {}", if prompt.is_empty() { ">" } else { "." }, prompt);
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        Ok(buf.trim().to_string())
    }

    fn fetch_idx() -> Result<usize, Box<dyn Error>> {
        let idx: usize = Cli::fetch("index: ")?.parse()?;
        Ok(idx)
    }

    fn build_pos() -> io::Result<PoS> {
        let name = Cli::fetch("name: ")?;
        let abbr = Cli::fetch("abbr: ")?;
        Ok(PoS::new(&name, &abbr))
    }

    fn execute_add_pos(&mut self) -> io::Result<()> {
        let item = Cli::build_pos()?;
        self.babel.add_pos(item);
        Ok(())
    }

    fn execute_alt_pos(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx()?;
        let item = Cli::build_pos()?;
        self.babel.alt_pos(idx, item)?;
        Ok(())
    }

    fn execute_debug(&self) {
        println!("{:#?}", self.babel);
    }

    fn execute_int(string: &str) {
        let interpreted = orth::interpret(string);
        println!("{}", interpreted);
    }

    fn execute_load(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let neo_babel = Babel::load(path)?;
        self.babel = neo_babel;
        println!("Loaded!");
        Ok(())
    }

    fn execute_ls_pos(&self) {
        for (i, pos) in self.babel.pos().iter().enumerate().filter(|&(_, x)| x.is_alive()) {
            println!("{}. {}({})", i, pos.name(), pos.abbr());
        }
    }
    fn execute_rm_pos(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx()?;
        self.babel.rm_pos(idx)?;
        Ok(())
    }

    fn execute_save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        self.babel.save(path)?;
        println!("Saved!");
        Ok(())
    }

    fn step(&mut self) -> Result<bool, Box<dyn Error>> {
        let buf = Cli::fetch("")?;
        let mut iter = buf.split_whitespace();
        match iter.next().unwrap_or("") {
            "add" => match iter.next().unwrap_or("") {
                "pos" => self.execute_add_pos()?,
                _ => println!("Unknown command")
            }
            "alt" => match iter.next().unwrap_or("") {
                "pos" => self.execute_alt_pos()?,
                _ => println!("Unknown command")
            }
            "dbg" => self.execute_debug(),
            "exit" | ";" => return Ok(false),
            "int" => Cli::execute_int(iter.next().unwrap_or("")),
            "load" => self.execute_load(iter.next().unwrap_or("project/example.json"))?,
            "ls" => match iter.next().unwrap_or("") {
                "pos" => self.execute_ls_pos(),
                _ => println!("Unknown command")
            }
            "rm" | "del" => match iter.next().unwrap_or("") {
                "pos" => self.execute_rm_pos()?,
                _ => println!("Unknown command")
            }
            "save" => self.execute_save(iter.next().unwrap_or("project/example.json"))?,
            "" => (),
            _ => println!("Unknown command")
        }
        Ok(true)
    }

    pub fn mainloop(&mut self) {
        loop {
            match self.step() {
                Ok(false) => break,
                Ok(true) => (),
                Err(err) => println!("Error occurred: {}", err),
            }
        }
    }
}