use std::env;

use crate::{
    strategy::Strategy,
    text,
    types::{CreateResponse, Output, Response},
};

use anyhow::{Result, anyhow};

use url::Url;

use reqwest::Client as HttpClient;

pub struct Options {
    pub api_key: String,

    pub api_version: String,

    pub resource: String,
}

pub struct Azure {
    api_key: String,

    url: Url,

    http_client: HttpClient,
}

impl Azure {
    pub fn from_env() -> Self {
        let api_key = env::var("AZURE_OPENAI_API_KEY").unwrap();

        let api_version = env::var("AZURE_OPENAI_API_VERSION").unwrap();

        let resource = env::var("AZURE_OPENAI_RESOURCE").unwrap();

        let options = Options {
            api_key,

            api_version,

            resource,
        };

        options.try_into().unwrap()
    }

    pub async fn text(&self, options: text::Options) -> Result<text::Response> {
        text::text(self, options).await
    }
}

impl TryFrom<Options> for Azure {
    type Error = url::ParseError;

    fn try_from(options: Options) -> Result<Self, Self::Error> {
        let Options {
            api_key,

            api_version,

            resource,
        } = options;

        let url_string = format!(
            "https://{resource}.openai.azure.com/openai/responses?api-version={api_version}"
        );

        let url = Url::parse(&url_string)?;

        let http_client = HttpClient::new();

        let result = Self {
            api_key,

            url,

            http_client,
        };

        Ok(result)
    }
}

impl Strategy for Azure {
    async fn create_response(&self, create_response: &CreateResponse) -> Result<Vec<Output>> {
        let Self {
            api_key,

            url,

            http_client,
        } = self;

        let response = http_client
            //
            .post(url.clone())
            //
            .header("api-key", api_key)
            //
            .json(&create_response)
            //
            .send()
            //
            .await?;

        let Response { error, output } = response
            //
            .json()
            //
            .await?;

        match (error, output) {
            (Some(e), _) => Err(anyhow!("error code: {}, message: {}", e.code, e.message)),

            (None, None) => Err(anyhow!("neither error nor output was provided")),

            (None, Some(output)) => Ok(output),
        }
    }
}
