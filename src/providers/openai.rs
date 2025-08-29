use std::env;
use crate::{
    error::{Error, Result},
    provider::{Provider, ProviderBuilder},
    types::{CreateResponse, Output, Response as ApiResponse},
};
use reqwest::Client as HttpClient;
use url::Url;

#[derive(Clone, Debug)]
pub struct OpenAIConfig {
    pub api_key: String,
}

impl OpenAIConfig {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| Error::Config("OPENAI_API_KEY environment variable not set".to_string()))?;
        
        Ok(Self { api_key })
    }
}

#[derive(Debug)]
pub struct OpenAIProvider {
    config: OpenAIConfig,
    url: Url,
    http_client: HttpClient,
}

impl OpenAIProvider {
    pub fn new(config: OpenAIConfig) -> Result<Self> {
        let url = Url::parse("https://api.openai.com/v1/responses")?;
        let http_client = HttpClient::new();
        
        Ok(Self {
            config,
            url,
            http_client,
        })
    }
    
    pub fn from_env() -> Result<Self> {
        let config = OpenAIConfig::from_env()?;
        Self::new(config)
    }
}

impl Provider for OpenAIProvider {
    type Config = OpenAIConfig;
    
    async fn create_response(&self, create_response: &CreateResponse) -> Result<Vec<Output>> {
        let response = self.http_client
            .post(self.url.clone())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
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
        "openai"
    }
}

#[derive(Debug)]
pub struct OpenAIBuilder {
    config: Option<OpenAIConfig>,
}

impl OpenAIBuilder {
    pub fn new() -> Self {
        Self { config: None }
    }
    
    pub fn with_config(mut self, config: OpenAIConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    pub fn from_env(mut self) -> Result<Self> {
        self.config = Some(OpenAIConfig::from_env()?);
        Ok(self)
    }
    
    pub fn api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.config = Some(OpenAIConfig {
            api_key: api_key.into(),
        });
        self
    }
}

impl ProviderBuilder<OpenAIProvider> for OpenAIBuilder {
    type Error = Error;
    
    fn build(self) -> Result<OpenAIProvider> {
        let config = self.config.ok_or_else(|| {
            Error::Config("OpenAI configuration not provided".to_string())
        })?;
        
        OpenAIProvider::new(config)
    }
}

impl Default for OpenAIBuilder {
    fn default() -> Self {
        Self::new()
    }
}