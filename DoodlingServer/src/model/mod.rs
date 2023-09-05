use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct DoodleEntry
{
    pub name: String,
    pub description: String,
    pub data: String
}