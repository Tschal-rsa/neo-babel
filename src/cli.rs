use crate::core::Babel;
use crate::core::language::{Language, Replace, SoundChange};
use crate::core::orth;
use crate::core::pos::PoS;
use crate::core::word::Word;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum CliError {
    InvalidInput,
    LanguageInvalid,
    Modified,
    UnknownCommand,
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CliError::InvalidInput => write!(f, "Invalid input!"),
            CliError::LanguageInvalid => write!(f, "You should create a language first."),
            CliError::Modified => write!(f, "You should save first."),
            CliError::UnknownCommand => write!(f, "Unknown command."),
        }
    }
}

impl Error for CliError {}

pub struct Cli {
    babel: Babel,
    cur_lang: Option<usize>,
    modified: bool,
}

impl Cli {
    pub fn new() -> Cli {
        Cli {
            babel: Babel::new(),
            cur_lang: None,
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

    fn fetch_int(prompt: &str) -> io::Result<String> {
        Cli::prompt(prompt);
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let int = orth::interpret(buf.trim());
        Cli::promptln(prompt, &int);
        Ok(int)
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

    fn fetch_int_or(prompt: &str, default: &str) -> io::Result<String> {
        Cli::promptln(prompt, &format!("(old) {}", default));
        Cli::fetch_int(prompt).map(|x| {
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

    fn fetch_char(prompt: &str) -> Result<char, Box<dyn Error>> {
        let name = Cli::fetch(prompt)?.chars().next().ok_or(CliError::InvalidInput)?;
        Ok(name)
    }

    fn build_new_lang(&self) -> io::Result<Language> {
        let name = Cli::fetch("name")?;
        Ok(Language::new(&name))
    }

    // fn build_lang() -> Result<Language, Box<dyn Error>> {
    //     let name = Cli::fetch("name")?;
    //     let ancestor = Cli::fetch_idx("ancestor's index")?;
    //     Ok(Language::new(&name, ancestor))
    // }

    fn update_lang(old: &mut Language) -> io::Result<()> {
        let name = Cli::fetch_or("name", old.name())?;
        old.change_name(&name);
        Ok(())
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

    fn build_replace() -> Result<Replace, Box<dyn Error>> {
        let pat = Cli::fetch_int("pattern")?;
        let repl = Cli::fetch_int("repl")?;
        let rule = Replace::new(&pat, &repl)?;
        Ok(rule)
    }

    fn update_replace(old: &Replace) -> Result<Replace, Box<dyn Error>> {
        let pat = Cli::fetch_int_or("pattern", old.pat())?;
        let repl = Cli::fetch_int_or("repl", old.repl())?;
        let rule = Replace::new(&pat, &repl)?;
        Ok(rule)
    }

    fn build_sound_change() -> io::Result<SoundChange> {
        let tg = Cli::fetch_int("target")?;
        let repl = Cli::fetch_int("repl")?;
        let env = Cli::fetch_int("env")?;
        Ok(SoundChange::new(&tg, &repl, &env))
    }

    fn update_sound_change(old: &SoundChange) -> io::Result<SoundChange> {
        let tg = Cli::fetch_int_or("target", old.tg())?;
        let repl = Cli::fetch_int_or("repl", old.repl())?;
        let env = Cli::fetch_int_or("env", old.env())?;
        Ok(SoundChange::new(&tg, &repl, &env))
    }

    fn build_word(&self) -> io::Result<Word> {
        let mnemonic = Cli::fetch_int("mnemonic")?;
        let natlang = Cli::fetch("natlang")?;
        let pos = loop {
            let abbr = Cli::fetch("pos")?;
            if let Some(idx) = self.babel.abbr_to_idx(&abbr) {
                break idx;
            }
        };
        let info = Cli::fetch("info")?;
        Ok(Word::shell(&mnemonic, &natlang, pos, &info))
    }

    fn update_word(&self, old: &Word) -> Result<Word, Box<dyn Error>> {
        let mnemonic = Cli::fetch_int_or("mnemonic", old.mnemonic())?;
        let natlang = Cli::fetch_or("natlang", old.natlang())?;
        let old_pos = self.babel.pos_at(old.pos())?.abbr();
        let pos = loop {
            let abbr = Cli::fetch_or("pos", old_pos)?;
            if let Some(idx) = self.babel.abbr_to_idx(&abbr) {
                break idx;
            }
        };
        let info = Cli::fetch_or("info", old.info())?;
        Ok(Word::shell(&mnemonic, &natlang, pos, &info))
    }

    fn check_lang(&self) -> Result<usize, CliError> {
        self.cur_lang.ok_or(CliError::LanguageInvalid)
    }

    fn cur_lang(&self) -> Result<&Language, Box<dyn Error>> {
        let cur_lang = self.check_lang()?;
        let lang = self.babel.lang_at(cur_lang)?;
        Ok(lang)
    }

    fn cur_lang_mut(&mut self) -> Result<&mut Language, Box<dyn Error>> {
        let cur_lang = self.check_lang()?;
        let lang = self.babel.lang_at_mut(cur_lang)?;
        Ok(lang)
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

    fn execute_add_lang(&mut self) -> io::Result<()> {
        let item = self.build_new_lang()?;
        self.babel.add_lang(item);
        self.modify();
        self.cur_lang = Some(self.babel.lang().len() - 1);
        Ok(())
    }

    fn execute_add_m2u(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang_mut()?;
        let item = Cli::build_replace()?;
        lang.add_m2u(item);
        self.modify();
        Ok(())
    }

    fn execute_add_m2w(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang_mut()?;
        let item = Cli::build_replace()?;
        lang.add_m2w(item);
        self.modify();
        Ok(())
    }

    fn execute_add_cat(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang_mut()?;
        let name = Cli::fetch_char("name")?;
        let content = Cli::fetch_int("content")?;
        lang.add_cat(name, &content);
        self.modify();
        Ok(())
    }

    fn execute_add_mnt(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang_mut()?;
        let sc = Cli::build_sound_change()?;
        lang.add_mnt(sc)?;
        self.modify();
        Ok(())
    }

    fn execute_add_pos(&mut self) -> io::Result<()> {
        let item = Cli::build_pos()?;
        self.babel.add_pos(item);
        self.modify();
        Ok(())
    }

    fn execute_add_word(&mut self) -> Result<(), Box<dyn Error>> {
        let word = self.build_word()?;
        let lang = self.cur_lang_mut()?;
        lang.add_word(word);
        Ok(())
    }

    fn execute_alt_lang(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        let old = self.babel.lang_at_mut(idx)?;
        Cli::update_lang(old)?;
        self.modify();
        self.cur_lang = Some(idx);
        Ok(())
    }

    fn execute_alt_m2u(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang()?;
        let idx = Cli::fetch_idx("index")?;
        let old = lang.m2u_at(idx)?;
        let item = Cli::update_replace(old)?;
        self.cur_lang_mut()?.alt_m2u(idx, item)?;
        self.modify();
        Ok(())
    }

    fn execute_alt_m2w(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang()?;
        let idx = Cli::fetch_idx("index")?;
        let old = lang.m2w_at(idx)?;
        let item = Cli::update_replace(old)?;
        self.cur_lang_mut()?.alt_m2w(idx, item)?;
        self.modify();
        Ok(())
    }

    fn execute_alt_mnt(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang()?;
        let idx = Cli::fetch_idx("index")?;
        let old = lang.mnt_at(idx)?;
        let sc = Cli::update_sound_change(old)?;
        self.cur_lang_mut()?.alt_mnt(idx, sc)?;
        self.modify();
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

    fn execute_alt_word(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang()?;
        let idx = Cli::fetch_idx("index")?;
        let old = lang.word_at(idx)?;
        let item = self.update_word(old)?;
        self.cur_lang_mut()?.alt_word(idx, item)?;
        self.modify();
        Ok(())
    }

    fn execute_cd(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        let lang = self.babel.lang_at(idx)?;
        self.cur_lang = Some(idx);
        println!("{}. {}", idx, self.babel.summarize_lang(lang));
        Ok(())
    }

    fn execute_debug(&self) -> Result<(), Box<dyn Error>> {
        // println!("{:#?}", self.babel);
        let lang = self.cur_lang()?;
        let sca = lang.mnemonic_transform().compile_all()?;
        let mut mnemonic = Cli::fetch_int("mnemonic")?;
        for sub in &sca {
            mnemonic = sub.pat().replace_all(&mnemonic, sub.repl()).into_owned();
        }
        println!("{:#?}", sca);
        println!("{}", mnemonic);
        Ok(())
    }

    fn execute_derive(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.check_lang()?;
        let ancestor_idx = Cli::fetch_idx("ancestor's index")?;
        self.babel.derive(lang, ancestor_idx)?;
        self.modify();
        Ok(())
    }

    fn execute_ins_m2u(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang_mut()?;
        let idx = Cli::fetch_idx("index")?;
        let item = Cli::build_replace()?;
        lang.ins_m2u(idx, item)?;
        self.modify();
        Ok(())
    }

    fn execute_ins_m2w(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang_mut()?;
        let idx = Cli::fetch_idx("index")?;
        let item = Cli::build_replace()?;
        lang.ins_m2w(idx, item)?;
        self.modify();
        Ok(())
    }

    fn execute_ins_mnt(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang_mut()?;
        let idx = Cli::fetch_idx("index")?;
        let sc = Cli::build_sound_change()?;
        lang.ins_mnt(idx, sc)?;
        self.modify();
        Ok(())
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
            self.cur_lang = Some(0);
            let lang = self.cur_lang()?;
            println!("0. {}", self.babel.summarize_lang(lang));
        } else {
            println!("Loaded!");
        }
        Ok(())
    }

    fn execute_ls_lang(&self) {
        for (i, lang) in self.babel.enum_lang() {
            println!("{}. {}", i, self.babel.summarize_lang(lang));
        }
    }

    fn execute_ls_m2w(&self) -> Result<(), Box<dyn Error>> {
        for (i, rule) in self.cur_lang()?.enum_m2w() {
            println!("{}. {} -> {}", i, rule.pat(), rule.repl());
        }
        Ok(())
    }

    fn execute_ls_m2u(&self) -> Result<(), Box<dyn Error>> {
        for (i, rule) in self.cur_lang()?.enum_m2u() {
            println!("{}. {} -> {}", i, rule.pat(), rule.repl());
        }
        Ok(())
    }

    fn execute_ls_cat(&self) -> Result<(), Box<dyn Error>> {
        for (i, (&name, content)) in self.cur_lang()?.enum_cat() {
            println!("{}. {} = {}", i, name, content);
        }
        Ok(())
    }

    fn execute_ls_mnt(&self) -> Result<(), Box<dyn Error>> {
        for (i, rule) in self.cur_lang()?.enum_mnt() {
            println!("{}.\t{:4} ->  {:4} /  {}", i, rule.tg(), rule.repl(), rule.env());
        }
        Ok(())
    }

    fn execute_ls_pos(&self) {
        for (i, pos) in self.babel.enum_pos() {
            println!("{}. {}({})", i, pos.name(), pos.abbr());
        }
    }

    fn execute_ls_word(&self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang()?;
        for (i, word) in lang.enum_word() {
            println!("{}.\t{}", i, self.babel.summarize_word(word));
        }
        Ok(())
    }

    fn execute_pwd(&self) -> Result<(), Box<dyn Error>> {
        let lang = self.cur_lang()?;
        println!("{}. {}", self.cur_lang.unwrap(), self.babel.summarize_lang(lang));
        Ok(())
    }

    fn execute_rm_lang(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        self.babel.rm_lang(idx)?;
        self.modify();
        if self.cur_lang == Some(idx) {
            self.cur_lang = if self.babel.lang().len() > 0 { Some(0) } else { None };
        }
        Ok(())
    }

    fn execute_rm_m2u(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        self.cur_lang_mut()?.rm_m2u(idx)?;
        self.modify();
        Ok(())
    }

    fn execute_rm_m2w(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        self.cur_lang_mut()?.rm_m2w(idx)?;
        self.modify();
        Ok(())
    }

    fn execute_rm_cat(&mut self) -> Result<(), Box<dyn Error>> {
        let name = Cli::fetch_char("name")?;
        let content = self.cur_lang_mut()?.rm_cat(name)?;
        println!("{}", content);
        self.modify();
        Ok(())
    }

    fn execute_rm_mnt(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        self.cur_lang_mut()?.rm_mnt(idx)?;
        self.modify();
        Ok(())
    }

    fn execute_rm_pos(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        self.babel.rm_pos(idx)?;
        self.modify();
        Ok(())
    }

    fn execute_rm_word(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        self.cur_lang_mut()?.rm_word(idx)?;
        self.modify();
        Ok(())
    }

    fn execute_rst_lang(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        let item = self.build_new_lang()?;
        self.babel.rst_lang(idx, item)?;
        self.modify();
        self.cur_lang = Some(idx);
        Ok(())
    }

    fn execute_rst_pos(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        let item = Cli::build_pos()?;
        self.babel.alt_pos(idx, item)?;
        self.modify();
        Ok(())
    }

    fn execute_rst_word(&mut self) -> Result<(), Box<dyn Error>> {
        let idx = Cli::fetch_idx("index")?;
        let item = self.build_word()?;
        self.cur_lang_mut()?.alt_word(idx, item)?;
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
                "m2u" => self.execute_add_m2u()?,
                "m2w" => self.execute_add_m2w()?,
                "cat" => self.execute_add_cat()?,
                "mnt" => self.execute_add_mnt()?,
                "pos" => self.execute_add_pos()?,
                "word" => self.execute_add_word()?,
                _ => return Err(Box::new(CliError::UnknownCommand))
            }
            "alt" => match iter.next().unwrap_or("") {
                "lang" => self.execute_alt_lang()?,
                "m2u" => self.execute_alt_m2u()?,
                "m2w" => self.execute_alt_m2w()?,
                "mnt" => self.execute_alt_mnt()?,
                "pos" => self.execute_alt_pos()?,
                "word" => self.execute_alt_word()?,
                _ => return Err(Box::new(CliError::UnknownCommand))
            }
            "cd" => self.execute_cd()?,
            "dbg" => self.execute_debug()?,
            "drv" => self.execute_derive()?,
            "q" | ";" => {
                self.check_modified()?;
                return Ok(false);
            }
            "q!" => return Ok(false),
            "ins" => match iter.next().unwrap_or("") {
                "m2u" => self.execute_ins_m2u()?,
                "m2w" => self.execute_ins_m2w()?,
                "mnt" => self.execute_ins_mnt()?,
                _ => return Err(Box::new(CliError::UnknownCommand))
            }
            "int" => Cli::execute_int(iter.next().unwrap_or("")),
            "load" => self.execute_load(iter.next().unwrap_or("project/example.json"))?,
            "ls" => match iter.next().unwrap_or("word") {
                "lang" => self.execute_ls_lang(),
                "m2w" => self.execute_ls_m2w()?,
                "m2u" => self.execute_ls_m2u()?,
                "cat" => self.execute_ls_cat()?,
                "mnt" => self.execute_ls_mnt()?,
                "pos" => self.execute_ls_pos(),
                "word" => self.execute_ls_word()?,
                _ => return Err(Box::new(CliError::UnknownCommand))
            }
            "pwd" => self.execute_pwd()?,
            "rm" | "del" => match iter.next().unwrap_or("") {
                "lang" => self.execute_rm_lang()?,
                "m2u" => self.execute_rm_m2u()?,
                "m2w" => self.execute_rm_m2w()?,
                "cat" => self.execute_rm_cat()?,
                "mnt" => self.execute_rm_mnt()?,
                "pos" => self.execute_rm_pos()?,
                "word" => self.execute_rm_word()?,
                _ => return Err(Box::new(CliError::UnknownCommand))
            }
            "rst" => match iter.next().unwrap_or("") {
                "lang" => self.execute_rst_lang()?,
                "pos" => self.execute_rst_pos()?,
                "word" => self.execute_rst_word()?,
                _ => return Err(Box::new(CliError::UnknownCommand))
            }
            "save" => self.execute_save(iter.next().unwrap_or("project/example.json"))?,
            "" => (),
            _ => return Err(Box::new(CliError::UnknownCommand))
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

impl Babel {
    fn summarize_lang(&self, lang: &Language) -> String {
        format!("{}({})", lang.name(), match lang.ancestor() {
            Some(ancestor) => match self.lang_at(ancestor) {
                Ok(x) => x.name(),
                Err(_) => "?",
            }
            None => "root",
        })
    }

    fn summarize_word(&self, word: &Word) -> String {
        let pos = match self.pos_at(word.pos()) {
            Ok(x) => x.abbr(),
            Err(_) => "?",
        };
        format!("{:10}\t{:5}\t{:20}", word.conlang(), pos, word.natlang())
    }
}