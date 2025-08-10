use crate::{
    Refusal, Response,
    types::{Output, OutputMessage, OutputMessageContent, Role},
};

use anyhow::{Result, ensure};

pub fn output_to_response(output: Vec<Output>) -> Result<Response<String>> {
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

        (Some(value), None) => Some(Ok(value)),

        (None, Some(value)) => Some(Err(<String as Into<Refusal>>::into(value))),

        _ => unreachable!(),
    };

    let response = Response {
        message,

        function_calls,
    };

    Ok(response)
}
