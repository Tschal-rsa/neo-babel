use serde::{Deserialize, Serialize};
// use serde_json::Result as JsonResult;
use super::Valid;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct Ancestor {
    lang: usize,
    word: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Word {
    conlang: String,
    natlang: String,
    pos: usize,
    upa: String,
    mnemonic: String,
    ancestor: Vec<Ancestor>,
    info: String,
}

impl Word {
    pub fn new(
        conlang: &str,
        natlang: &str,
        pos: usize,
        upa: &str,
        mnemonic: &str,
        ancestor: &Vec<Ancestor>,
        info: &str
    ) -> Word {
        Word {
            conlang: conlang.to_string(),
            natlang: natlang.to_string(),
            pos,
            upa: upa.to_string(),
            mnemonic: mnemonic.to_string(),
            ancestor: ancestor.clone(),
            info: info.to_string()
        }
    }

    pub fn construct(
        conlang: String,
        natlang: String,
        pos: usize,
        upa: String,
        mnemonic: String,
        ancestor: Vec<Ancestor>,
        info: String
    ) -> Word {
        Word { conlang, natlang, pos, upa, mnemonic, ancestor, info }
    }
}

impl Valid for Word {
    fn destroy(&mut self) {
        self.conlang.clear();
        self.natlang.clear();
        self.pos = usize::MAX;
        self.upa.clear();
        self.mnemonic.clear();
        self.ancestor.clear();
        self.info.clear();
    }

    fn is_alive(&self) -> bool {
        !self.conlang.is_empty()
    }
}