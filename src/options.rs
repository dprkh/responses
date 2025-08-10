use crate::types::{Input, Tool, ToolChoice};

#[derive(Clone, Debug, Default)]
pub struct Options {
    pub safety_identifier: Option<String>,

    pub model: Option<String>,

    pub tools: Option<Vec<Tool>>,

    pub tool_choice: Option<ToolChoice>,

    pub input: Option<Vec<Input>>,
}
