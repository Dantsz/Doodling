pub(crate) use async_trait::async_trait;
use surrealdb::{Surreal, engine::remote::ws::{Client, Ws}, opt::auth::{Credentials, self}};
use anyhow::Result;

use crate::model::DoodleEntry;

#[async_trait]
pub trait DoodleDataStore : Clone + Send + Sync
{
    async fn get_recent_doodles(&self) -> Result<Vec<DoodleEntry>>;
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
    async fn get_recent_doodles(&self) -> Result<Vec<DoodleEntry>>
    {
        Ok(self.surreal_client
        .select("Doodles")
        .await?)
    }

    async fn create_doodle(&self, doodle: DoodleEntry) -> Result<()>
    {
        self.surreal_client.create::<Vec<DoodleEntry>>("Doodles").content::<DoodleEntry>(doodle).await?;
        Ok(())
    }
}