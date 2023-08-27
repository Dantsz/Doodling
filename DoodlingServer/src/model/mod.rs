use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct DoodleEntry
{
    name: String,
    description: String
}