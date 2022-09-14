use serde::{Deserialize, Serialize};
// use serde_json::Result as JsonResult;
use regex::{self, Regex};
use std::collections::HashMap;
use std::error::Error;
use super::{Babel, BabelError};
use super::word::{Word, Coordinate};

#[derive(Deserialize, Serialize, Debug)]
pub struct SoundChange {
    tg: String,
    repl: String,
    env: String,
}

impl SoundChange {
    pub fn new(tg: &str, repl: &str, env: &str) -> SoundChange {
        SoundChange { tg: tg.to_string(), repl: repl.to_string(), env: env.to_string() }
    }

    pub fn tg(&self) -> &str {
        &self.tg
    }

    pub fn repl(&self) -> &str {
        &self.repl
    }

    pub fn env(&self) -> &str {
        &self.env
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SCA {
    cat: HashMap<char, String>,
    sc: Vec<SoundChange>,
}

impl SCA {
    pub fn new() -> SCA {
        SCA { cat: HashMap::new(), sc: Vec::new() }
    }

    pub fn cat(&self) -> &HashMap<char, String> {
        &self.cat
    }

    pub fn sc(&self) -> &Vec<SoundChange> {
        &self.sc
    }

    pub fn add_cat(&mut self, name: char, content: &str) {
        self.cat.insert(name, content.to_string());
    }

    pub fn add_sc(&mut self, sc: SoundChange) -> Result<(), Box<dyn Error>> {
        self.compile(&sc)?;
        self.sc.push(sc);
        Ok(())
    }

    pub fn alt_sc(&mut self, idx: usize, sc: SoundChange) -> Result<(), Box<dyn Error>> {
        self.compile(&sc)?;
        Language::template_alt(&mut self.sc, idx, sc)?;
        Ok(())
    }

    pub fn ins_sc(&mut self, idx: usize, sc: SoundChange) -> Result<(), Box<dyn Error>> {
        self.compile(&sc)?;
        Language::template_ins(&mut self.sc, idx, sc)?;
        Ok(())
    }

    pub fn rm_cat(&mut self, name: char) -> Result<String, BabelError> {
        self.cat.remove(&name).ok_or(BabelError::IndexOutOfRange)
    }

    pub fn rm_sc(&mut self, idx: usize) -> Result<(), BabelError> {
        Language::template_rm(&mut self.sc, idx)
    }

    fn compile_unit(&self, sc: &SoundChange) -> Result<Substitute, Box<dyn Error>> {
        let env: Vec<_> = sc.env().split('_').collect();
        let mut pat = format!(
            "(?P<pre>{}){}(?P<post>{})",
            env.get(0).ok_or(BabelError::InvalidSCEnvironment)?,
            sc.tg(),
            env.get(1).ok_or(BabelError::InvalidSCEnvironment)?
        );
        for (name, content) in self.cat.iter() {
            pat = pat.replace(&name.to_string(), &format!("[{}]", content));
        }
        let repl = format!("${{pre}}{}${{post}}", sc.repl());
        let sub = Substitute::new(&pat, &repl)?;
        Ok(sub)
    }

    pub fn compile(&self, sc: &SoundChange) -> Result<Vec<Substitute>, Box<dyn Error>> {
        let repl_contains_key = sc.repl().chars().any(|x| self.cat.contains_key(&x));
        if repl_contains_key {
            let min_tg = sc.tg().chars().filter_map(|x| {
                self.cat.get(&x).map(|s| utf8_slice::len(s))
            }).min().ok_or(BabelError::InvalidSCTarget)?;
            let min_repl = sc.repl().chars().filter_map(|x| {
                self.cat.get(&x).map(|s| utf8_slice::len(s))
            }).min().unwrap();
            let min_len = if min_tg < min_repl { min_tg } else { min_repl };
            let mut subset = Vec::new();
            for idx in 0..min_len {
                let mut tg = sc.tg().to_owned();
                let mut repl = sc.repl().to_owned();
                for (name, content) in self.cat.iter() {
                    tg = tg.replace(&name.to_string(), utf8_slice::slice(content, idx, idx + 1));
                    repl = repl.replace(&name.to_string(), utf8_slice::slice(content, idx, idx + 1));
                }
                let sc = SoundChange::new(&tg, &repl, sc.env());
                let sub = self.compile_unit(&sc)?;
                subset.push(sub);
            }
            Ok(subset)
        } else {
            let sub = self.compile_unit(sc)?;
            Ok(vec![sub])
        }
    }

    pub fn compile_all(&self) -> Result<Vec<Substitute>, Box<dyn Error>> {
        let mut set = Vec::new();
        for sc in self.sc.iter() {
            let mut subset = self.compile(sc)?;
            set.append(&mut subset);
        }
        Ok(set)
    }
}

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

impl Substitute {
    pub fn new(pat: &str, repl: &str) -> Result<Substitute, regex::Error> {
        let pat = Regex::new(pat)?;
        Ok(Substitute { pat, repl: repl.to_owned() })
    }
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
    vocab: Vec<Option<Word>>,
    mnemonic_to_word: Vec<Replace>,
    mnemonic_to_upa: Vec<Replace>,
    mnemonic_transform: SCA,
}

impl Language {
    pub fn new(name: &str) -> Language {
        Language {
            name: name.to_string(),
            ancestor: None,
            vocab: Vec::new(),
            mnemonic_to_word: Vec::new(),
            mnemonic_to_upa: Vec::new(),
            mnemonic_transform: SCA::new(),
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

    pub fn mnemonic_transform(&self) -> &SCA {
        &self.mnemonic_transform
    }

    pub fn m2w_at(&self, idx: usize) -> Result<&Replace, BabelError> {
        Language::template_at(&self.mnemonic_to_word, idx)
    }

    pub fn m2u_at(&self, idx: usize) -> Result<&Replace, BabelError> {
        Language::template_at(&self.mnemonic_to_upa, idx)
    }

    pub fn cat_at(&self, name: char) -> Result<&String, BabelError> {
        self.mnemonic_transform.cat().get(&name).ok_or(BabelError::IndexOutOfRange)
    }

    pub fn mnt_at(&self, idx: usize) -> Result<&SoundChange, BabelError> {
        Language::template_at(self.mnemonic_transform.sc(), idx)
    }

    pub fn word_at(&self, idx: usize) -> Result<&Word, BabelError> {
        Babel::template_at(&self.vocab, idx)
    }

    pub fn word_at_mut(&mut self, idx: usize) -> Result<&mut Word, BabelError> {
        Babel::template_at_mut(&mut self.vocab, idx)
    }

    fn make_m2w(&self) -> Vec<Substitute> {
        self.mnemonic_to_word.iter().map(|x| Substitute::from(x)).collect()
    }

    fn make_m2u(&self) -> Vec<Substitute> {
        self.mnemonic_to_upa.iter().map(|x| Substitute::from(x)).collect()
    }

    fn make_mnt(&self) -> Vec<Substitute> {
        self.mnemonic_transform.compile_all().unwrap()
    }

    pub fn add_word(&mut self, mut word: Word) {
        let m2w = self.make_m2w();
        let m2u = self.make_m2u();
        word.morph(&m2w, &m2u);
        self.vocab.push(Some(word));
    }

    pub fn add_m2w(&mut self, item: Replace) {
        Language::template_add(&mut self.mnemonic_to_word, item);
    }

    pub fn add_m2u(&mut self, item: Replace) {
        Language::template_add(&mut self.mnemonic_to_upa, item);
    }

    pub fn add_cat(&mut self, name: char, content: &str) {
        self.mnemonic_transform.add_cat(name, content);
    }

    pub fn add_mnt(&mut self, item: SoundChange) -> Result<(), Box<dyn Error>> {
        self.mnemonic_transform.add_sc(item)
    }

    pub fn alt_m2w(&mut self, idx: usize, item: Replace) -> Result<(), BabelError> {
        Language::template_alt(&mut self.mnemonic_to_word, idx, item)
    }

    pub fn alt_m2u(&mut self, idx: usize, item: Replace) -> Result<(), BabelError> {
        Language::template_alt(&mut self.mnemonic_to_upa, idx, item)
    }

    pub fn alt_mnt(&mut self, idx: usize, item: SoundChange) -> Result<(), Box<dyn Error>> {
        self.mnemonic_transform.alt_sc(idx, item)
    }

    pub fn alt_word(&mut self, idx: usize, mut item: Word) -> Result<(), BabelError> {
        let m2w = self.make_m2w();
        let m2u = self.make_m2u();
        item.morph(&m2w, &m2u);
        let old_ancestor = self.vocab.get(idx).ok_or(BabelError::IndexOutOfRange)?.as_ref().ok_or(BabelError::InvalidElement)?.ancestor();
        item.set_ancestor(old_ancestor);
        Babel::template_alt(&mut self.vocab, idx, item)
    }

    pub fn drv(&mut self, ancestor_idx: usize, ancestor: &Language) -> Result<(), BabelError> {
        let m2w = self.make_m2w();
        let m2u = self.make_m2u();
        let mnt = self.make_mnt();
        self.ancestor = Some(ancestor_idx);
        let mut queue: Vec<_> = ancestor.vocab.iter().map(|x| x.as_ref()).collect();
        for (idx, word) in self.enum_word_mut() {
            let ancestor_coord = word.ancestor();
            if ancestor_coord.len() == 1 && ancestor_coord[0].lang() == ancestor_idx {
                let ancestor_coord = ancestor_coord[0];
                let word_ancestor = queue.get(ancestor_coord.word()).ok_or(BabelError::GhostWord(idx))?.ok_or(BabelError::GhostWord(idx))?;
                let neo_word = word_ancestor.labor(ancestor_coord, &mnt, &m2w, &m2u);
                word.fuse(neo_word);
                queue[ancestor_coord.word()] = None;
            }
        }
        for (idx, word_option) in queue.iter().enumerate() {
            if let Some(word_ancestor) = *word_option {
                let ancestor_coord = Coordinate::new(ancestor_idx, idx);
                self.vocab.push(Some(word_ancestor.labor(ancestor_coord, &mnt, &m2w, &m2u)));
            }
        }
        Ok(())
    }

    pub fn enum_m2w(&self) -> impl Iterator<Item = (usize, &Replace)> {
        Language::template_enum(&self.mnemonic_to_word)
    }

    pub fn enum_m2u(&self) -> impl Iterator<Item = (usize, &Replace)> {
        Language::template_enum(&self.mnemonic_to_upa)
    }

    pub fn enum_cat(&self) -> impl Iterator<Item = (usize, (&char, &String))> {
        self.mnemonic_transform.cat().iter().enumerate()
    }

    pub fn enum_mnt(&self) -> impl Iterator<Item = (usize, &SoundChange)> {
        Language::template_enum(self.mnemonic_transform.sc())
    }

    pub fn enum_word(&self) -> impl Iterator<Item = (usize, &Word)> {
        Babel::template_enum(&self.vocab)
    }

    fn enum_word_mut(&mut self) -> impl Iterator<Item = (usize, &mut Word)> {
        Babel::template_enum_mut(&mut self.vocab)
    }

    pub fn etym_word(&mut self, idx: usize, ancestors: &[Coordinate]) -> Result<(), BabelError> {
        let word = self.vocab.get_mut(idx).ok_or(BabelError::IndexOutOfRange)?.as_mut().ok_or(BabelError::InvalidElement)?;
        word.set_ancestor(ancestors);
        Ok(())
    }

    pub fn ins_m2w(&mut self, idx: usize, item: Replace) -> Result<(), BabelError> {
        Language::template_ins(&mut self.mnemonic_to_word, idx, item)
    }

    pub fn ins_m2u(&mut self, idx: usize, item: Replace) -> Result<(), BabelError> {
        Language::template_ins(&mut self.mnemonic_to_upa, idx, item)
    }

    pub fn ins_mnt(&mut self, idx: usize, item: SoundChange) -> Result<(), Box<dyn Error>> {
        self.mnemonic_transform.ins_sc(idx, item)
    }

    pub fn revive(&mut self) {
        let m2w = self.make_m2w();
        let m2u = self.make_m2u();
        for word in self.vocab.iter_mut().filter_map(|x| x.as_mut()) {
            word.morph(&m2w, &m2u);
        }
    }

    pub fn rm_m2w(&mut self, idx: usize) -> Result<(), BabelError> {
        Language::template_rm(&mut self.mnemonic_to_word, idx)
    }

    pub fn rm_m2u(&mut self, idx: usize) -> Result<(), BabelError> {
        Language::template_rm(&mut self.mnemonic_to_upa, idx)
    }

    pub fn rm_cat(&mut self, name: char) -> Result<String, BabelError> {
        self.mnemonic_transform.rm_cat(name)
    }

    pub fn rm_mnt(&mut self, idx: usize) -> Result<(), BabelError> {
        self.mnemonic_transform.rm_sc(idx)
    }

    pub fn rm_word(&mut self, idx: usize) -> Result<(), BabelError> {
        Babel::template_rm(&mut self.vocab, idx)
    }
    
    pub fn rst_word(&mut self, idx: usize, mut item: Word) -> Result<(), BabelError> {
        let m2w = self.make_m2w();
        let m2u = self.make_m2u();
        item.morph(&m2w, &m2u);
        Babel::template_alt(&mut self.vocab, idx, item)
    }

    fn template_add<T>(seq: &mut Vec<T>, item: T) {
        seq.push(item);
    }

    fn template_alt<T>(seq: &mut Vec<T>, idx: usize, item: T) -> Result<(), BabelError> {
        let old_item = seq.get_mut(idx).ok_or(BabelError::IndexOutOfRange)?;
        *old_item = item;
        Ok(())
    }

    fn template_at<T>(seq: &Vec<T>, idx: usize) -> Result<&T, BabelError> {
        seq.get(idx).ok_or(BabelError::IndexOutOfRange)
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

// impl Valid for Language {
//     fn destroy(&mut self) {
//         self.name.clear();
//         self.ancestor = None;
//     }

//     fn is_alive(&self) -> bool {
//         !self.name.is_empty()
//     }
// }