use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct DoodleEntry
{
    id: String,
    name: String,
    description: String
}