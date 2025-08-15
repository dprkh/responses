use crate::providers::AzureBuilder;

pub struct ClientBuilder;

impl ClientBuilder {
    pub fn azure() -> AzureBuilder {
        AzureBuilder::new()
    }
}

impl AzureBuilder {
    // build_client method removed - use .build()? followed by Client::new(provider) instead
}

pub fn azure() -> AzureBuilder {
    ClientBuilder::azure()
}