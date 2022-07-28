use crate::core::Babel;
use crate::core::language::Language;
use crate::core::orth;
use crate::core::pos::PoS;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

const INVALID_LANG: usize = usize::MAX;

#[derive(Debug)]
pub enum CliError {
    LanguageInvalid,
    Modified,
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CliError::LanguageInvalid => write!(f, "You should create a language first."),
            CliError::Modified => write!(f, "You should save first."),
        }
    }
}

impl Error for CliError {}

pub struct Cli {
    babel: Babel,
    cur_lang: usize,
    modified: bool,
}

impl Cli {
    pub fn new() -> Cli {
        Cli {
            babel: Babel::new(),
            cur_lang: INVALID_LANG,
            modified: false,
        }
    }

    fn prompt(prompt: &str) {
        let line = if prompt.is_empty() {
            String::from("> ")
        } else {
            format!(". {}: ", prompt)
        };
        eprint!("{}", line);
    }

    fn promptln(prompt: &str, value: &str) {
        let line = if prompt.is_empty() {
            format!("> {}", value)
        } else {
            format!(". {}: {}", prompt, value)
        };
        eprintln!("{}", line);
    }

    fn fetch(prompt: &str) -> io::Result<String> {
        Cli::prompt(prompt);
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        Ok(buf.trim().to_string())
    }

    fn fetch_or(prompt: &str, default: &str) -> io::Result<String> {
        Cli::promptln(prompt, &format!("(old) {}", default));
        Cli::fetch(prompt).map(|x| {
            if x.is_empty() {
                default.to_string()
            } else {
                x
            }
        })
    }

    fn fetch_idx(prompt: &str) -> Result<usize, Box<dyn Error>> {
        let idx: usize = Cli::fetch(prompt)?.parse()?;
        Ok(idx)
    }

    fn fetch_idx_or(prompt: &str, default: usize) -> Result<usize, Box<dyn Error>> {
        Cli::promptln(prompt, &format!("(old) {}", default));
        let x = Cli::fetch(prompt)?;
        let idx: usize = if x.is_empty() {
            default
        } else {
            x.parse()?
        };
        Ok(idx)
    }

    fn build_new_lang(&self) -> io::Result<Language> {
        let name = Cli::fetch("name")?;
        let ancestor = self.babel.lang().len();
        Ok(Language::new(&name, ancestor))
    }

    // fn build_lang() -> Result<Language, Box<dyn Error>> {
    //     let name = Cli::fetch("name")?;
    //     let ancestor = Cli::fetch_idx("ancestor's index")?;
    //     Ok(Language::new(&name, ancestor))
    // }

    fn update_lang(old: &Language) -> Result<Language, Box<dyn Error>> {
        let name = Cli::fetch_or("name", old.name())?;
        let ancestor = Cli::fetch_idx_or("ancestor's index", old.ancestor())?;
        Ok(Language::new(&name, ancestor))
    }

    fn build_pos() -> io::Result<PoS> {
        let name = Cli::fetch("name")?;
        let abbr = Cli::fetch("abbr")?;
        Ok(PoS::new(&name, &abbr))
    }

    fn update_pos(old: &PoS) -> io::Result<PoS> {
        let name = Cli::fetch_or("name", old.name())?;
        let abbr = Cli::fetch_or("abbr", old.abbr())?;
        Ok(PoS::new(&name, &abbr))
    }

    fn check_lang(&self) -> Result<(), CliError> {
        if self.cur_lang == INVALID_LANG {
            Err(CliError::LanguageInvalid)
        } else {
            Ok(())
        }
    }

    fn modify(&mut self) {
        self.modified = true;
    }

    fn check_modified(&self) -> Result<(), CliError> {
        if self.modified {
            Err(CliError::Modified)
        } else {
            Ok(())
        }
    }

    fn execute_add_lang(&mut self) -> Result<(), Box<dyn Error>> {
        let item = self.build_new_lang()?;
        self.babel.add_lang(item)?;
        self.modify();
        self.cur_lang = self.babel.lang().len() - 1;
        Ok(())
    }

    fn execute_add_pos(&mut self) -> Result<(), Box<dyn Error>> {
        let item = Cli::build_pos()?;
        self.babel.add_pos(item)?;
        self.modify();
        Ok(())
    }

    fn execute_alt_lang(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        let old = self.babel.lang_at(idx)?;
        let item = Cli::update_lang(old)?;
        self.babel.alt_lang(idx, item)?;
        self.modify();
        self.cur_lang = idx;
        Ok(())
    }

    fn execute_alt_pos(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        let old = self.babel.pos_at(idx)?;
        let item = Cli::update_pos(old)?;
        self.babel.alt_pos(idx, item)?;
        self.modify();
        Ok(())
    }

    fn execute_cd(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        let lang = self.babel.lang_at(idx)?;
        self.cur_lang = idx;
        println!("{}. {}({})", self.cur_lang, lang.name(), lang.ancestor());
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
        self.check_modified()?;
        let neo_babel = Babel::load(path)?;
        self.babel = neo_babel;
        if self.babel.lang().len() > 0 {
            self.cur_lang = 0;
        }
        println!("Loaded!");
        Ok(())
    }

    fn execute_ls_lang(&self) {
        for (i, lang) in self.babel.enum_lang() {
            println!("{}. {}({})", i, lang.name(), lang.ancestor());
        }
    }

    fn execute_ls_pos(&self) {
        for (i, pos) in self.babel.enum_pos() {
            println!("{}. {}({})", i, pos.name(), pos.abbr());
        }
    }

    fn execute_pwd(&self) -> Result<(), Box<dyn Error>> {
        self.check_lang()?;
        let lang = self.babel.lang_at(self.cur_lang)?;
        println!("{}. {}({})", self.cur_lang, lang.name(), lang.ancestor());
        Ok(())
    }

    fn execute_rm_lang(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        self.babel.rm_lang(idx)?;
        self.modify();
        if self.cur_lang == idx {
            self.cur_lang = if self.babel.lang().len() > 0 { 0 } else { INVALID_LANG };
        }
        Ok(())
    }

    fn execute_rm_pos(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        self.babel.rm_pos(idx)?;
        self.modify();
        Ok(())
    }

    fn execute_save(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        self.babel.save(path)?;
        self.modified = false;
        println!("Saved!");
        Ok(())
    }

    fn step(&mut self) -> Result<bool, Box<dyn Error>> {
        let buf = Cli::fetch("")?;
        let mut iter = buf.split_whitespace();
        match iter.next().unwrap_or("") {
            "add" => match iter.next().unwrap_or("") {
                "lang" => self.execute_add_lang()?,
                "pos" => self.execute_add_pos()?,
                _ => println!("Unknown command")
            }
            "alt" => match iter.next().unwrap_or("") {
                "lang" => self.execute_alt_lang()?,
                "pos" => self.execute_alt_pos()?,
                _ => println!("Unknown command")
            }
            "cd" => self.execute_cd()?,
            "dbg" => self.execute_debug(),
            "exit" | ";" => {
                self.check_modified()?;
                return Ok(false);
            }
            "!" => return Ok(false),
            "int" => Cli::execute_int(iter.next().unwrap_or("")),
            "load" => self.execute_load(iter.next().unwrap_or("project/example.json"))?,
            "ls" => match iter.next().unwrap_or("") {
                "lang" => self.execute_ls_lang(),
                "pos" => self.execute_ls_pos(),
                _ => println!("Unknown command")
            }
            "pwd" => self.execute_pwd()?,
            "rm" | "del" => match iter.next().unwrap_or("") {
                "lang" => self.execute_rm_lang()?,
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
            eprintln!("");
        }
    }
}