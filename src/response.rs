use crate::{Refusal, types::OutputFunctionCall};

/// Response from an API call containing text messages and/or function calls.
/// 
/// Both text responses and function calls are equally valid response types.
/// Function-only responses (where `message` is `None`) are normal behavior
/// when the model chooses to respond only with function calls.
#[derive(Clone, Debug)]
pub struct Response<T> {
    /// The text response from the model (can be `None` for function-only responses).
    pub message: Option<Result<T, Refusal>>,

    /// Function calls requested by the model.
    pub function_calls: Vec<OutputFunctionCall>,
}

impl<T> Response<T> {
    /// Returns true if the response contains a text message.
    /// 
    /// This helper clarifies that function-only responses are normal behavior,
    /// not error conditions.
    pub fn has_text_message(&self) -> bool {
        self.message.is_some()
    }

    /// Returns true if the response contains function calls.
    pub fn has_function_calls(&self) -> bool {
        !self.function_calls.is_empty()
    }

    /// Returns true if this is a function-only response (no text message).
    /// 
    /// Function-only responses are a normal interaction pattern where
    /// the model responds exclusively with function calls.
    pub fn is_function_only(&self) -> bool {
        self.message.is_none() && !self.function_calls.is_empty()
    }

    /// Returns true if this response contains both text and function calls.
    pub fn has_both_text_and_functions(&self) -> bool {
        self.has_text_message() && self.has_function_calls()
    }

    /// Get a reference to the text message if it exists and was successful.
    /// 
    /// Returns `None` for function-only responses or if the message was refused.
    pub fn text_message(&self) -> Option<&T> {
        self.message.as_ref().and_then(|result| result.as_ref().ok())
    }

    /// Get a reference to the refusal if the message was refused.
    pub fn refusal(&self) -> Option<&Refusal> {
        self.message.as_ref().and_then(|result| result.as_ref().err())
    }

    /// Get the number of function calls in this response.
    pub fn function_call_count(&self) -> usize {
        self.function_calls.len()
    }
}
