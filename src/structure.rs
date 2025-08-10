use crate::{
    Options, Response,
    strategy::Strategy,
    types::{CreateResponse, Text, TextFormat, TextFormatJsonSchema},
    util::output_to_response,
};

use anyhow::{Context, Result};

use schemars::{
    JsonSchema, Schema,
    generate::SchemaSettings,
    transform::{RecursiveTransform, ReplaceConstValue},
};

use serde::Deserialize;

pub async fn structure<S, T>(strategy: &S, name: String, options: Options) -> Result<Response<T>>
where
    S: Strategy,
    T: JsonSchema + for<'a> Deserialize<'a>,
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

        text: Some(Text {
            format: TextFormat::JsonSchema(TextFormatJsonSchema {
                name,

                schema: schema_for::<T>(),

                strict: true,

                ..Default::default()
            }),

            ..Default::default()
        }),

        tools,

        tool_choice,

        input,

        store: Some(false),

        ..Default::default()
    };

    let output = strategy.create_response(&create_response).await?;

    let Response {
        message,

        function_calls,
    } = output_to_response(output)?;

    let message = match message {
        Some(result) => match result {
            Ok(string) => {
                let message = serde_json::from_str::<T>(&string)
                    .context("failed to parse response message")?;

                Some(Ok(message))
            }

            Err(refusal) => Some(Err(refusal)),
        },

        None => None,
    };

    let response = Response {
        message,

        function_calls,
    };

    Ok(response)
}

fn schema_for<T: ?Sized + JsonSchema>() -> Schema {
    SchemaSettings::default()
        //
        .for_serialize()
        //
        .with(|s| s.meta_schema = None)
        //
        .with_transform(ReplaceConstValue::default())
        //
        .with_transform(RecursiveTransform(|schema: &mut Schema| {
            if schema.get("properties").is_some() {
                schema.insert("additionalProperties".to_owned(), false.into());
            }

            if let Some(one_of) = schema.remove("oneOf") {
                schema.insert("anyOf".to_owned(), one_of);
            }
        }))
        //
        .into_generator()
        //
        .into_root_schema_for::<T>()
}
