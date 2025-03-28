use serde::{Deserialize, Serialize};
use serde_json::value::Value;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Command<'r> {
    Select {
        database: &'r str,
    },
    List,
    Append {
        collection: &'r str,
        data: Value,  // TODO bug line-based property with RawValue?
    },
    Prepare {
        code: String,
    },
    Execute {
        code_ref: &'r str,
        query: &'r str,
        param: Value,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Collection {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CodeRef {
    pub code_ref: String,
}
