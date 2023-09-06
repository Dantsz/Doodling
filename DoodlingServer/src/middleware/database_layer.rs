use surrealdb::{Surreal, engine::remote::ws::Client};
use anyhow::Result;

use crate::model::DoodleEntry;


struct DbConnection{
    sureal_client: Surreal<Client>
}

impl DbConnection
{
    async fn new(connection_string: String) -> Self
    {
        unimplemented!()
    }

    async fn get_recent_doodles(&self) -> Vec<DoodleEntry>
    {
        unimplemented!();
    }

    async fn create_doodle(&mut self, doodle: DoodleEntry) -> Result<()>
    {
        unimplemented!();
    }
}