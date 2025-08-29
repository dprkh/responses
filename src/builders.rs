use crate::providers::{AzureBuilder, OpenAIBuilder};

pub struct ClientBuilder;

impl ClientBuilder {
    pub fn azure() -> AzureBuilder {
        AzureBuilder::new()
    }
    
    pub fn openai() -> OpenAIBuilder {
        OpenAIBuilder::new()
    }
}

impl AzureBuilder {
    // build_client method removed - use .build()? followed by Client::new(provider) instead
}

pub fn azure() -> AzureBuilder {
    ClientBuilder::azure()
}

pub fn openai() -> OpenAIBuilder {
    ClientBuilder::openai()
}