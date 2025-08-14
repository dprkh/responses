use crate::{
    error::{Error, Result},
    Refusal, Response,
    types::{Output, OutputMessage, OutputMessageContent, Role},
};

pub fn output_to_response(output: Vec<Output>) -> Result<Response<String>> {
    let mut text: Option<String> = None;

    let mut refusal: Option<String> = None;

    let mut function_calls = Vec::new();

    for item in output {
        match item {
            Output::Message(message) => {
                let OutputMessage { role, content } = message;

                if !matches!(role, Role::Assistant) {
                    return Err(Error::InvalidResponse(
                        "Output message role must be assistant".to_string()
                    ));
                }

                for item in content {
                    match item {
                        OutputMessageContent::OutputText(value) => {
                            if text.is_some() {
                                return Err(Error::InvalidResponse(
                                    "Output message content must contain at most one text".to_string()
                                ));
                            }

                            if refusal.is_some() {
                                return Err(Error::InvalidResponse(
                                    "Output message content must not contain text and refusal at the same time".to_string()
                                ));
                            }

                            text.replace(value.text);
                        }

                        OutputMessageContent::Refusal(value) => {
                            if refusal.is_some() {
                                return Err(Error::InvalidResponse(
                                    "Output message content must contain at most one refusal".to_string()
                                ));
                            }

                            if text.is_some() {
                                return Err(Error::InvalidResponse(
                                    "Output message content must not contain text and refusal at the same time".to_string()
                                ));
                            }

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
