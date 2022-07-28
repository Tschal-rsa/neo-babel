use serde::{Deserialize, Serialize};
// use serde_json::Result as JsonResult;
use super::Valid;
use super::word::Word;

#[derive(Deserialize, Serialize, Debug)]
pub struct Replace {
    pat: String,
    repl: String,
}

impl Replace {
    pub fn new(pat: &str, repl: &str) -> Replace {
        Replace { pat: pat.to_string(), repl: repl.to_string() }
    }

    pub fn pat(&self) -> &str {
        &self.pat
    }
    
    pub fn repl(&self) -> &str {
        &self.repl
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Language {
    name: String,
    ancestor: usize,
    vocab: Vec<Word>,
    mnemonic_to_word: Vec<Replace>
}

impl Language {
    pub fn new(name: &str, ancestor: usize) -> Language {
        Language {
            name: name.to_string(),
            ancestor,
            vocab: Vec::new(),
            mnemonic_to_word: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ancestor(&self) -> usize {
        self.ancestor
    }

    pub fn mnemonic_to_word(&self) -> &Vec<Replace> {
        &self.mnemonic_to_word
    }

    pub fn append_mnemonic_to_word(&mut self, item: Replace) {
        self.mnemonic_to_word.push(item);
    }

    pub fn insert_mnemonic_to_word(&mut self, idx: usize, item: Replace) {
        self.mnemonic_to_word.insert(idx, item);
    }

    pub fn remove_mnemonic_to_word(&mut self, idx: usize) {
        self.mnemonic_to_word.remove(idx);
    }

    pub fn update_mnemonic_to_word(&mut self, idx: usize, item: Replace) {
        self.mnemonic_to_word[idx] = item;
    }
}

impl Valid for Language {
    fn destroy(&mut self) {
        self.name.clear();
        self.ancestor = usize::MAX;
    }

    fn is_alive(&self) -> bool {
        !self.name.is_empty()
    }
}