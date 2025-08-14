use crate::{
    client::Client,
    error::Result,
    providers::{AzureProvider, AzureBuilder},
    provider::ProviderBuilder,
};

pub struct ClientBuilder;

impl ClientBuilder {
    pub fn azure() -> AzureBuilder {
        AzureBuilder::new()
    }
}

impl AzureBuilder {
    pub fn build_client(self) -> Result<Client<AzureProvider>> {
        let provider = self.build()?;
        Ok(Client::new(provider))
    }
}

pub fn azure() -> AzureBuilder {
    ClientBuilder::azure()
}