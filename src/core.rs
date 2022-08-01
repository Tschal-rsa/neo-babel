pub mod class;
pub mod language;
pub mod orth;
pub mod pos;
pub mod word;

use language::Language;
use pos::PoS;
use serde::{Deserialize, Serialize};
// use serde_json::Result as JsonResult;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::File;

// trait Valid {
//     fn destroy(&mut self);
//     fn is_alive(&self) -> bool;
// }

#[derive(Debug)]
pub enum BabelError {
    // AdditionRejected,
    // AlterationRejected,
    DeriveFromSelf,
    GhostWord(usize),
    IndexOutOfRange,
    // InvalidCatagory(char),
    InvalidElement,
    InvalidSCEnvironment,
    InvalidSCTarget,
}

impl Display for BabelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            // BabelError::AdditionRejected => write!(f, "Addition is rejected."),
            // BabelError::AlterationRejected => write!(f, "Alteration is rejected."),
            BabelError::DeriveFromSelf => write!(f, "Cannot derive from self!"),
            BabelError::GhostWord(idx) => write!(f, "Ghost word: {}", idx),
            BabelError::IndexOutOfRange => write!(f, "Index out of range!"),
            // BabelError::InvalidCatagory(name) => write!(f, "Invalid catagory: {}", name),
            BabelError::InvalidElement => write!(f, "Invalid element!"),
            BabelError::InvalidSCEnvironment => write!(f, "Invalid SC environment!"),
            BabelError::InvalidSCTarget => write!(f, "Invalid SC target!"),
        }
    }
}

impl Error for BabelError {}

#[derive(Deserialize, Serialize, Debug)]
pub struct Babel {
    language: Vec<Option<Language>>,
    pos: Vec<Option<PoS>>,
}

impl Babel {
    pub fn new() -> Babel {
        Babel {
            language: Vec::new(),
            pos: Vec::new(),
        }
    }

    pub fn lang(&self) -> &Vec<Option<Language>> {
        &self.language
    }

    pub fn pos(&self) -> &Vec<Option<PoS>> {
        &self.pos
    }

    pub fn lang_at(&self, idx: usize) -> Result<&Language, BabelError> {
        Babel::template_at(&self.language, idx)
    }

    pub fn lang_at_mut(&mut self, idx: usize) -> Result<&mut Language, BabelError> {
        Babel::template_at_mut(&mut self.language, idx)
    }

    pub fn pos_at(&self, idx: usize) -> Result<&PoS, BabelError> {
        Babel::template_at(&self.pos, idx)
    }

    pub fn abbr_to_idx(&self, abbr: &str) -> Option<usize> {
        for (i, pos) in self.enum_pos() {
            if pos.abbr() == abbr {
                return Some(i);
            }
        }
        None
    }

    pub fn add_lang(&mut self, item: Language) {
        Babel::template_add(&mut self.language, item);
    }

    pub fn add_pos(&mut self, item: PoS) {
        Babel::template_add(&mut self.pos, item);
    }

    // pub fn alt_lang(&mut self, idx: usize, item: Language) -> Result<(), BabelError> {
    //     Babel::template_alt(&mut self.language, idx, item)
    // }

    pub fn rst_lang(&mut self, idx: usize, item: Language) -> Result<(), BabelError> {
        Babel::template_alt(&mut self.language, idx, item)
    }

    pub fn alt_pos(&mut self, idx: usize, item: PoS) -> Result<(), BabelError> {
        Babel::template_alt(&mut self.pos, idx, item)
    }

    pub fn derive(&mut self, lang: usize, ancestor_idx: usize) -> Result<(), BabelError> {
        if lang == ancestor_idx {
            return Err(BabelError::DeriveFromSelf);
        } else if lang >= self.language.len() || ancestor_idx >= self.language.len() {
            return Err(BabelError::IndexOutOfRange);
        }
        let (lang, ancestor) = if lang > ancestor_idx {
            let (first, second) = self.language.split_at_mut(lang);
            (&mut second[0], &first[ancestor_idx])
        } else {
            let (first, second) = self.language.split_at_mut(ancestor_idx);
            (&mut first[lang], &second[0])
        };
        let lang = lang.as_mut().ok_or(BabelError::InvalidElement)?;
        let ancestor = ancestor.as_ref().ok_or(BabelError::InvalidElement)?;
        lang.drv(ancestor_idx, ancestor)?;
        Ok(())
    }

    pub fn enum_lang(&self) -> impl Iterator<Item = (usize, &Language)> {
        Babel::template_enum(&self.language)
    }

    pub fn enum_pos(&self) -> impl Iterator<Item = (usize, &PoS)> {
        Babel::template_enum(&self.pos)
    }

    pub fn load(path: &str) -> Result<Babel, Box<dyn Error>> {
        let file = File::open(path)?;
        let neo_babel: Babel = serde_json::from_reader(file)?;
        Ok(neo_babel)
    }

    pub fn rm_lang(&mut self, idx: usize) -> Result<(), BabelError> {
        Babel::template_rm(&mut self.language, idx)
    }

    pub fn rm_pos(&mut self, idx: usize) -> Result<(), BabelError> {
        Babel::template_rm(&mut self.pos, idx)
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    // pub fn add_m2w(&mut self, lang: usize, item: Replace) -> Result<(), BabelError> {
    //     let lang = self.lang_at_mut(lang)?;
    //     lang.append_mnemonic_to_word(item);
    //     Ok(())
    // }

    // pub fn alt_m2w(&mut self, lang: usize, idx: usize, item: Replace) -> Result<(), BabelError> {
    //     let lang = self.lang_at_mut(lang)?;
    //     if idx >= lang.mnemonic_to_word().len() {
    //         return Err(BabelError::IndexOutOfRange);
    //     }
    //     lang.update_mnemonic_to_word(idx, item);
    //     Ok(())
    // }

    // pub fn ins_m2w(&mut self, lang: usize, idx: usize, item: Replace) -> Result<(), BabelError> {
    //     let lang = self.lang_at_mut(lang)?;
    //     if idx > lang.mnemonic_to_word().len() {
    //         return Err(BabelError::IndexOutOfRange);
    //     }
    //     lang.insert_mnemonic_to_word(idx, item);
    //     Ok(())
    // }

    // pub fn rm_m2w(&mut self, lang: usize, idx: usize) -> Result<(), BabelError> {
    //     let lang = self.lang_at_mut(lang)?;
    //     if idx >= lang.mnemonic_to_word().len() {
    //         return Err(BabelError::IndexOutOfRange);
    //     }
    //     lang.remove_mnemonic_to_word(idx);
    //     Ok(())
    // }

    fn template_add<T>(seq: &mut Vec<Option<T>>, item: T) {
        seq.push(Some(item))
    }

    fn template_alt<T>(seq: &mut Vec<Option<T>>, idx: usize, item: T) -> Result<(), BabelError> {
        let old_item = seq.get_mut(idx).ok_or(BabelError::IndexOutOfRange)?;
        *old_item = Some(item);
        Ok(())
    }

    fn template_at<T>(seq: &Vec<Option<T>>, idx: usize) -> Result<&T, BabelError> {
        let item = seq.get(idx).ok_or(BabelError::IndexOutOfRange)?;
        let item = item.as_ref().ok_or(BabelError::InvalidElement)?;
        Ok(item)
    }

    fn template_at_mut<T>(seq: &mut Vec<Option<T>>, idx: usize) -> Result<&mut T, BabelError> {
        let item = seq.get_mut(idx).ok_or(BabelError::IndexOutOfRange)?;
        let item = item.as_mut().ok_or(BabelError::InvalidElement)?;
        Ok(item)
    }

    fn template_enum<T>(seq: &Vec<Option<T>>) -> impl Iterator<Item = (usize, &T)> {
        seq.iter().enumerate().filter_map(|(idx, item)| {
            match item.as_ref() {
                Some(x) => Some((idx, x)),
                None => None
            }
        })
    }

    fn template_enum_mut<T>(seq: &mut Vec<Option<T>>) -> impl Iterator<Item = (usize, &mut T)> {
        seq.iter_mut().enumerate().filter_map(|(idx, item)| {
            match item.as_mut() {
                Some(x) => Some((idx, x)),
                None => None
            }
        })
    }

    fn template_rm<T>(seq: &mut Vec<Option<T>>, idx: usize) -> Result<(), BabelError> {
        let old_item = seq.get_mut(idx).ok_or(BabelError::IndexOutOfRange)?;
        *old_item = None;
        Ok(())
    }
}