use serde::{Deserialize, Serialize};
// use serde_json::Result as JsonResult;

#[derive(Deserialize, Serialize, Debug)]
struct Class {
    name: String,
    apply: Vec<usize>,
    values: Vec<String>,
}

impl Class {

}