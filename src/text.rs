use crate::{
    Options, Response, strategy::Strategy, types::CreateResponse, util::output_to_response,
};

use anyhow::Result;

pub async fn text<S>(strategy: &S, options: Options) -> Result<Response<String>>
where
    S: Strategy,
{
    let Options {
        safety_identifier,

        model,

        tools,

        tool_choice,

        input,
    } = options;

    let create_response = CreateResponse {
        safety_identifier,

        model,

        tools,

        tool_choice,

        input,

        store: Some(false),

        ..Default::default()
    };

    let output = strategy.create_response(&create_response).await?;

    let response = output_to_response(output)?;

    Ok(response)
}
