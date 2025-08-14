use std::env;
use crate::{
    error::{Error, Result},
    provider::{Provider, ProviderBuilder},
    types::{CreateResponse, Output, Response as ApiResponse},
};
use reqwest::Client as HttpClient;
use url::Url;

#[derive(Clone, Debug)]
pub struct AzureConfig {
    pub api_key: String,
    pub resource: String,
    pub api_version: String,
}

impl AzureConfig {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("AZURE_OPENAI_API_KEY")
            .map_err(|_| Error::Config("AZURE_OPENAI_API_KEY environment variable not set".to_string()))?;
        
        let resource = env::var("AZURE_OPENAI_RESOURCE")
            .map_err(|_| Error::Config("AZURE_OPENAI_RESOURCE environment variable not set".to_string()))?;
        
        let api_version = env::var("AZURE_OPENAI_API_VERSION")
            .map_err(|_| Error::Config("AZURE_OPENAI_API_VERSION environment variable not set".to_string()))?;
        
        Ok(Self {
            api_key,
            resource,
            api_version,
        })
    }
}

pub struct AzureProvider {
    config: AzureConfig,
    url: Url,
    http_client: HttpClient,
}

impl AzureProvider {
    pub fn new(config: AzureConfig) -> Result<Self> {
        let url_string = format!(
            "https://{}.openai.azure.com/openai/responses?api-version={}",
            config.resource, config.api_version
        );
        
        let url = Url::parse(&url_string)?;
        let http_client = HttpClient::new();
        
        Ok(Self {
            config,
            url,
            http_client,
        })
    }
    
    pub fn from_env() -> Result<Self> {
        let config = AzureConfig::from_env()?;
        Self::new(config)
    }
}

impl Provider for AzureProvider {
    type Config = AzureConfig;
    
    async fn create_response(&self, create_response: &CreateResponse) -> Result<Vec<Output>> {
        let response = self.http_client
            .post(self.url.clone())
            .header("api-key", &self.config.api_key)
            .json(&create_response)
            .send()
            .await?;
        
        let api_response: ApiResponse = response.json().await?;
        
        match (api_response.error, api_response.output) {
            (Some(e), _) => Err(Error::Provider {
                code: e.code,
                message: e.message,
            }),
            (None, None) => Err(Error::InvalidResponse(
                "Neither error nor output was provided".to_string()
            )),
            (None, Some(output)) => Ok(output),
        }
    }
    
    fn name(&self) -> &'static str {
        "azure"
    }
}

pub struct AzureBuilder {
    config: Option<AzureConfig>,
}

impl AzureBuilder {
    pub fn new() -> Self {
        Self { config: None }
    }
    
    pub fn with_config(mut self, config: AzureConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    pub fn from_env(mut self) -> Result<Self> {
        self.config = Some(AzureConfig::from_env()?);
        Ok(self)
    }
    
    pub fn api_key<S: Into<String>>(self, api_key: S) -> AzureConfigBuilder {
        AzureConfigBuilder {
            api_key: Some(api_key.into()),
            resource: None,
            api_version: None,
        }
    }
}

impl ProviderBuilder<AzureProvider> for AzureBuilder {
    type Error = Error;
    
    fn build(self) -> Result<AzureProvider> {
        let config = self.config.ok_or_else(|| {
            Error::Config("Azure configuration not provided".to_string())
        })?;
        
        AzureProvider::new(config)
    }
}

pub struct AzureConfigBuilder {
    api_key: Option<String>,
    resource: Option<String>,
    api_version: Option<String>,
}

impl AzureConfigBuilder {
    pub fn resource<S: Into<String>>(mut self, resource: S) -> Self {
        self.resource = Some(resource.into());
        self
    }
    
    pub fn api_version<S: Into<String>>(mut self, api_version: S) -> Self {
        self.api_version = Some(api_version.into());
        self
    }
    
    pub fn build(self) -> Result<AzureBuilder> {
        let api_key = self.api_key.ok_or_else(|| {
            Error::Config("API key is required".to_string())
        })?;
        
        let resource = self.resource.ok_or_else(|| {
            Error::Config("Resource name is required".to_string())
        })?;
        
        let api_version = self.api_version.unwrap_or_else(|| "2024-02-15-preview".to_string());
        
        let config = AzureConfig {
            api_key,
            resource,
            api_version,
        };
        
        Ok(AzureBuilder::new().with_config(config))
    }
}

impl Default for AzureBuilder {
    fn default() -> Self {
        Self::new()
    }
}