use serde::{Deserialize, Serialize};
// use serde_json::Result as JsonResult;

#[derive(Deserialize, Serialize, Debug)]
pub struct PoS {
    name: String,
    abbr: String,
}

impl PoS {
    pub fn new(name: &str, abbr: &str) -> PoS {
        PoS { name: name.to_string(), abbr: abbr.to_string() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn abbr(&self) -> &str {
        &self.abbr
    }
}

// impl Valid for PoS {
//     fn destroy(&mut self) {
//         self.name.clear();
//         self.abbr.clear();
//     }

//     fn is_alive(&self) -> bool {
//         !self.name.is_empty()
//     }
// }