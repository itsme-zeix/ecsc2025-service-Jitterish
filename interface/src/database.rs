use serde::{Deserialize, Serialize};
use serde_json::value::Value;

#[derive(Serialize, Deserialize)]
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
        code: &'r str,
    },
    Execute {
        code_ref: &'r str,
        query: &'r str,
        param: Value,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Collection<'r> {
    pub name: &'r str,
}
