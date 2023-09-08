pub(crate) use async_trait::async_trait;
use surrealdb::{Surreal, engine::remote::ws::{Client, Ws}, opt::auth::{Credentials, self}};
use anyhow::Result;

use crate::model::DoodleEntry;

#[async_trait]
pub trait DoodleDataStore : Clone + Send + Sync
{
    async fn get_recent_doodles(&self,limit: usize) -> Result<Vec<DoodleEntry>>;
    async fn create_doodle(&self, doodle: DoodleEntry) -> Result<()>;
}

#[derive(Clone)]
pub struct SurrealDoodleConnection{
    surreal_client: Surreal<Client>
}

impl SurrealDoodleConnection
{
    pub async fn new(client : Surreal<Client>) -> Self
    {
        Self{
            surreal_client: client
        }
    }
}

#[async_trait]
impl DoodleDataStore for SurrealDoodleConnection
{
    async fn get_recent_doodles(&self,limit: usize) -> Result<Vec<DoodleEntry>>
    {
        Ok(self.surreal_client
        .query("SELECT * FROM Doodles LIMIT $limit")
        .bind(("limit",limit))
        .await?
        .take(0)?
        )
    }

    async fn create_doodle(&self, doodle: DoodleEntry) -> Result<()>
    {
        self.surreal_client.create::<Vec<DoodleEntry>>("Doodles").content::<DoodleEntry>(doodle).await?;
        Ok(())
    }
}