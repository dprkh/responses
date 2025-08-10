use crate::{Refusal, types::OutputFunctionCall};

#[derive(Clone, Debug)]
pub struct Response<T> {
    pub message: Option<Result<T, Refusal>>,

    pub function_calls: Vec<OutputFunctionCall>,
}
