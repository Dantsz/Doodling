
struct DbConnection{
    sureal_client: Client<Sureal>
}

impl DnConnection
{
    async fn new(connection_string: String) -> Self
    {

    }

    async fn get_recent_doodles(&self) -> Vec<DoodleEntry>
    {
        unimplemented!();
    }

    async fn create_doodle(&mut self, doodle: DoodleEntry) -> Result<(),Error>
    {
        unimplemented!();
    }
}