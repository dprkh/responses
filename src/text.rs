use crate::{
    strategy::Strategy,
    types::{
        CreateResponse, Input, Output, OutputFunctionCall, OutputMessage, OutputMessageContent,
        Role, Tool, ToolChoice,
    },
};

use anyhow::{Result, ensure};

#[derive(Clone, Debug, Default)]
pub struct Options {
    pub safety_identifier: Option<String>,

    pub model: Option<String>,

    pub tools: Option<Vec<Tool>>,

    pub tool_choice: Option<ToolChoice>,

    pub input: Option<Vec<Input>>,
}

#[derive(Clone, Debug)]
pub struct Response {
    pub message: Option<Message>,

    pub function_calls: Vec<OutputFunctionCall>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Text(String),

    Refusal(String),
}

pub async fn text<S>(strategy: &S, options: Options) -> Result<Response>
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

    let mut text: Option<String> = None;

    let mut refusal: Option<String> = None;

    let mut function_calls = Vec::new();

    for item in output {
        match item {
            Output::Message(message) => {
                let OutputMessage { role, content } = message;

                ensure!(
                    matches!(role, Role::Assistant),
                    "output message role must be assistant"
                );

                for item in content {
                    match item {
                        OutputMessageContent::OutputText(value) => {
                            ensure!(
                                text.is_none(),
                                "output message content must contain at most one text"
                            );

                            ensure!(
                                refusal.is_none(),
                                "output message content must not contain text and refusal at the same time"
                            );

                            text.replace(value.text);
                        }

                        OutputMessageContent::Refusal(value) => {
                            ensure!(
                                refusal.is_none(),
                                "output message content must contain at most one refusal"
                            );

                            ensure!(
                                text.is_none(),
                                "output message content must not contain text and refusal at the same time"
                            );

                            refusal.replace(value.refusal);
                        }
                    }
                }
            }

            Output::FunctionCall(function_call) => {
                function_calls.push(function_call);
            }
        }
    }

    let message = match (text, refusal) {
        (None, None) => None,

        (Some(text), None) => Some(Message::Text(text)),

        (None, Some(refusal)) => Some(Message::Refusal(refusal)),

        _ => unreachable!(),
    };

    let response = Response {
        message,

        function_calls,
    };

    Ok(response)
}
