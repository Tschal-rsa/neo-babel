use serde::{Deserialize, Serialize};
// use serde_json::Result as JsonResult;
use regex::Regex;
use std::borrow::Cow;
use super::language::Substitute;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct Coordinate {
    lang: usize,
    word: usize,
}

impl Coordinate {
    pub fn new(lang: usize, word: usize) -> Self {
        Self { lang, word }
    }

    pub fn lang(&self) -> usize {
        self.lang
    }

    pub fn word(&self) -> usize {
        self.word
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Word {
    conlang: String,
    natlang: String,
    pos: usize,
    upa: String,
    mnemonic: String,
    ancestor: Vec<Coordinate>,
    info: String,
}

impl Word {
    pub fn shell(
        mnemonic: &str,
        natlang: &str,
        pos: usize,
        info: &str
    ) -> Word {
        Word {
            conlang: String::new(),
            natlang: natlang.to_string(),
            pos,
            upa: String::new(),
            mnemonic: mnemonic.to_string(),
            ancestor: Vec::new(),
            info: info.to_string()
        }
    }

    pub fn conlang(&self) -> &str {
        &self.conlang
    }

    pub fn natlang(&self) -> &str {
        &self.natlang
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }

    pub fn ancestor(&self) -> &Vec<Coordinate> {
        &self.ancestor
    }

    pub fn add_ancestor(&mut self, coord: Coordinate) {
        self.ancestor.push(coord);
    }

    pub fn set_ancestor(&mut self, other: &[Coordinate]) {
        self.ancestor = other.to_vec();
    }

    pub fn info(&self) -> &str {
        &self.info
    }

    fn replace_all(re: &Regex, text: &str, rep: &str) -> String {
        let mut text = text.to_owned();
        loop {
            match re.replace_all(&text, rep) {
                Cow::Borrowed(_) => break text,
                Cow::Owned(s) => text = s,
            }
        }
    }

    pub fn fuse(&mut self, other: Word) {
        self.conlang = other.conlang;
        self.upa = other.upa;
        self.mnemonic = other.mnemonic;
    }

    pub fn morph(&mut self, m2w: &Vec<Substitute>, m2u: &Vec<Substitute>) {
        let mut conlang = self.mnemonic.to_owned();
        for sub in m2w {
            conlang = Word::replace_all(sub.pat(), &conlang, sub.repl());
        }
        self.conlang = conlang;
        let mut upa = self.mnemonic.to_owned();
        for sub in m2u {
            upa = Word::replace_all(sub.pat(), &upa, sub.repl());
        }
        self.upa = upa;
    }

    pub fn labor(&self, coord: Coordinate, mnt: &Vec<Substitute>, m2w: &Vec<Substitute>, m2u: &Vec<Substitute>) -> Word {
        let mut mnemonic = self.mnemonic.to_owned();
        for sub in mnt {
            mnemonic = Word::replace_all(sub.pat(), &mnemonic, sub.repl());
        }
        let mut word = Self::shell(&mnemonic, &self.natlang, self.pos, &self.info);
        word.ancestor.push(coord);
        word.morph(m2w, m2u);
        word
    }

    // pub fn new(
    //     conlang: &str,
    //     natlang: &str,
    //     pos: usize,
    //     upa: &str,
    //     mnemonic: &str,
    //     ancestor: &Vec<Coordinate>,
    //     info: &str
    // ) -> Word {
    //     Word {
    //         conlang: conlang.to_string(),
    //         natlang: natlang.to_string(),
    //         pos,
    //         upa: upa.to_string(),
    //         mnemonic: mnemonic.to_string(),
    //         ancestor: ancestor.clone(),
    //         info: info.to_string()
    //     }
    // }

    // pub fn construct(
    //     conlang: String,
    //     natlang: String,
    //     pos: usize,
    //     upa: String,
    //     mnemonic: String,
    //     ancestor: Vec<Coordinate>,
    //     info: String
    // ) -> Word {
    //     Word { conlang, natlang, pos, upa, mnemonic, ancestor, info }
    // }
}

// impl Valid for Word {
//     fn destroy(&mut self) {
//         self.conlang.clear();
//         self.natlang.clear();
//         self.pos = usize::MAX;
//         self.upa.clear();
//         self.mnemonic.clear();
//         self.ancestor.clear();
//         self.info.clear();
//     }

//     fn is_alive(&self) -> bool {
//         !self.conlang.is_empty()
//     }
// }