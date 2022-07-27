pub mod class;
pub mod orth;
pub mod pos;

use pos::PoS;
use serde::{Deserialize, Serialize};
// use serde_json::Result as JsonResult;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::File;

pub trait Valid {
    fn destroy(&mut self);
    fn is_alive(&self) -> bool;
}

#[derive(Debug)]
pub enum BabelError {
    AlterationRejected,
    IndexOutOfRange,
}

impl Display for BabelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BabelError::AlterationRejected => write!(f, "Alteration is rejected."),
            BabelError::IndexOutOfRange => write!(f, "Index out of range!"),
        }
    }
}

impl Error for BabelError {}

#[derive(Deserialize, Serialize, Debug)]
pub struct Babel {
    pos: Vec<PoS>
}

impl Babel {
    pub fn new() -> Babel {
        Babel {
            pos: Vec::new()
        }
    }

    pub fn add_pos(&mut self, item: PoS) {
        self.pos.push(item);
    }

    pub fn alt_pos(&mut self, idx: usize, item: PoS) -> Result<(), BabelError> {
        if !item.is_alive() {
            return Err(BabelError::AlterationRejected);
        }
        let old_item = self.pos.get_mut(idx).ok_or(BabelError::IndexOutOfRange)?;
        *old_item = item;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Babel, Box<dyn Error>> {
        let file = File::open(path)?;
        let neo_babel: Babel = serde_json::from_reader(file)?;
        Ok(neo_babel)
    }

    pub fn pos(&self) -> &Vec<PoS> {
        &self.pos
    }

    pub fn rm_pos(&mut self, idx: usize) -> Result<(), BabelError> {
        let old_item = self.pos.get_mut(idx).ok_or(BabelError::IndexOutOfRange)?;
        old_item.destroy();
        Ok(())
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }
}