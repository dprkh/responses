use crate::{
    error::Result, 
    provider::Provider,
    response::Response,
    types::CreateResponse,
    util::output_to_response,
    request::{TextRequestBuilder, StructuredRequestBuilder},
    Options,
};
use schemars::JsonSchema;
use serde::Deserialize;

pub struct Client<P: Provider> {
    provider: P,
}

impl<P: Provider> Client<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }
    
    pub fn text(&self) -> TextRequestBuilder<'_, P> {
        TextRequestBuilder::new(self)
    }
    
    pub fn structured<T>(&self) -> StructuredRequestBuilder<'_, P, T>
    where
        T: JsonSchema + for<'a> Deserialize<'a>,
    {
        let name = std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap_or("Response")
            .to_string();
        
        StructuredRequestBuilder::new(self, name)
    }
    
    pub fn structured_with_name<T>(&self, name: String) -> StructuredRequestBuilder<'_, P, T>
    where
        T: JsonSchema + for<'a> Deserialize<'a>,
    {
        StructuredRequestBuilder::new(self, name)
    }
    

    pub async fn text_with_options(&self, options: Options) -> Result<Response<String>> {
        let create_response = CreateResponse {
            safety_identifier: options.safety_identifier,
            model: options.model,
            tools: options.tools,
            tool_choice: options.tool_choice,
            input: options.input,
            store: Some(false),
            text: None,
        };
        
        let output = self.provider.create_response(&create_response).await?;
        let response = output_to_response(output)?;
        Ok(response)
    }
    
    pub async fn structure_with_name_and_options<T>(&self, name: String, options: Options) -> Result<Response<T>>
    where
        T: JsonSchema + for<'a> Deserialize<'a>,
    {
        use crate::{
            schema,
            types::{Text, TextFormat, TextFormatJsonSchema},
        };
        
        let create_response = CreateResponse {
            safety_identifier: options.safety_identifier,
            model: options.model,
            text: Some(Text {
                format: TextFormat::JsonSchema(TextFormatJsonSchema {
                    name,
                    schema: schema::from::<T>(),
                    strict: true,
                    description: None,
                }),
                verbosity: None,
            }),
            tools: options.tools,
            tool_choice: options.tool_choice,
            input: options.input,
            store: Some(false),
        };
        
        let output = self.provider.create_response(&create_response).await?;
        
        let Response { message, function_calls } = output_to_response(output)?;
        
        let message = match message {
            Some(result) => match result {
                Ok(string) => {
                    let parsed = serde_json::from_str::<T>(&string)
                        .map_err(crate::error::Error::Json)?;
                    Some(Ok(parsed))
                }
                Err(refusal) => Some(Err(refusal)),
            },
            None => None,
        };
        
        Ok(Response { message, function_calls })
    }
    
    pub fn provider(&self) -> &P {
        &self.provider
    }
}