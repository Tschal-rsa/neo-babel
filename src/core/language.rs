use serde::{Deserialize, Serialize};
// use serde_json::Result as JsonResult;
use regex::{self, Regex};
use super::{BabelError, Valid};
use super::word::Word;

#[derive(Deserialize, Serialize, Debug)]
pub struct Replace {
    pat: String,
    repl: String,
}

impl Replace {
    pub fn new(pat: &str, repl: &str) -> Result<Replace, regex::Error> {
        Regex::new(pat)?;
        Ok(Replace { pat: pat.to_string(), repl: repl.to_string() })
    }

    pub fn pat(&self) -> &str {
        &self.pat
    }
    
    pub fn repl(&self) -> &str {
        &self.repl
    }
}

#[derive(Debug)]
pub struct Substitute {
    pat: Regex,
    repl: String,
}

impl From<&Replace> for Substitute {
    fn from(original: &Replace) -> Self {
        Self { pat: Regex::new(original.pat()).unwrap(), repl: original.repl().to_owned() }
    }
}

impl Substitute {
    pub fn pat(&self) -> &Regex {
        &self.pat
    }

    pub fn repl(&self) -> &str {
        &self.repl
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Language {
    name: String,
    ancestor: Option<usize>,
    vocab: Vec<Word>,
    mnemonic_to_word: Vec<Replace>,
    mnemonic_to_upa: Vec<Replace>,
}

impl Language {
    pub fn new(name: &str) -> Language {
        Language {
            name: name.to_string(),
            ancestor: None,
            vocab: Vec::new(),
            mnemonic_to_word: Vec::new(),
            mnemonic_to_upa: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn change_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    pub fn ancestor(&self) -> Option<usize> {
        self.ancestor
    }

    pub fn display_ancestor(&self) -> String {
        match self.ancestor {
            Some(ancestor) => ancestor.to_string(),
            None => String::from("root")
        }
    }

    pub fn mnemonic_to_word(&self) -> &Vec<Replace> {
        &self.mnemonic_to_word
    }

    fn make_m2w(&self) -> Vec<Substitute> {
        self.mnemonic_to_word.iter().map(|x| Substitute::from(x)).collect()
    }

    fn make_m2u(&self) -> Vec<Substitute> {
        self.mnemonic_to_upa.iter().map(|x| Substitute::from(x)).collect()
    }

    pub fn add_word(&mut self, mut word: Word) {
        let m2w = self.make_m2w();
        let m2u = self.make_m2u();
        word.morph(&m2w, &m2u);
        self.vocab.push(word);
    }

    pub fn add_m2w(&mut self, item: Replace) {
        Language::template_add(&mut self.mnemonic_to_word, item);
    }

    pub fn alt_m2w(&mut self, idx: usize, item: Replace) -> Result<(), BabelError> {
        Language::template_alt(&mut self.mnemonic_to_word, idx, item)
    }

    pub fn enum_m2w(&self) -> impl Iterator<Item = (usize, &Replace)> {
        Language::template_enum(&self.mnemonic_to_word)
    }

    pub fn ins_m2w(&mut self, idx: usize, item: Replace) -> Result<(), BabelError> {
        Language::template_ins(&mut self.mnemonic_to_word, idx, item)
    }

    pub fn rm_m2w(&mut self, idx: usize) -> Result<(), BabelError> {
        Language::template_rm(&mut self.mnemonic_to_word, idx)
    }

    fn template_add<T>(seq: &mut Vec<T>, item: T) {
        seq.push(item);
    }

    fn template_alt<T>(seq: &mut Vec<T>, idx: usize, item: T) -> Result<(), BabelError> {
        if idx >= seq.len() {
            return Err(BabelError::IndexOutOfRange);
        }
        seq[idx] = item;
        Ok(())
    }

    fn template_enum<T>(seq: &Vec<T>) -> impl Iterator<Item = (usize, &T)> {
        seq.iter().enumerate()
    }

    fn template_ins<T>(seq: &mut Vec<T>, idx: usize, item: T) -> Result<(), BabelError> {
        if idx > seq.len() {
            return Err(BabelError::IndexOutOfRange);
        }
        seq.insert(idx, item);
        Ok(())
    }

    fn template_rm<T>(seq: &mut Vec<T>, idx: usize) -> Result<(), BabelError> {
        if idx >= seq.len() {
            return Err(BabelError::IndexOutOfRange);
        }
        seq.remove(idx);
        Ok(())
    }

    // pub(super) fn append_mnemonic_to_word(&mut self, item: Replace) {
    //     self.mnemonic_to_word.push(item);
    // }

    // pub(super) fn insert_mnemonic_to_word(&mut self, idx: usize, item: Replace) {
    //     self.mnemonic_to_word.insert(idx, item);
    // }

    // pub(super) fn remove_mnemonic_to_word(&mut self, idx: usize) {
    //     self.mnemonic_to_word.remove(idx);
    // }

    // pub(super) fn update_mnemonic_to_word(&mut self, idx: usize, item: Replace) {
    //     self.mnemonic_to_word[idx] = item;
    // }
}

impl Valid for Language {
    fn destroy(&mut self) {
        self.name.clear();
        self.ancestor = None;
    }

    fn is_alive(&self) -> bool {
        !self.name.is_empty()
    }
}