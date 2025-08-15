use serde::{Deserialize, Serialize};

use schemars::Schema;

#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Vec<Input>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
}

#[derive(Copy, Clone, Debug, Serialize)]
pub enum Verbosity {
    #[serde(rename = "low")]
    Low,

    #[serde(rename = "medium")]
    Medium,

    #[serde(rename = "high")]
    High,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Text {
    pub format: TextFormat,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<Verbosity>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(tag = "type")]
pub enum TextFormat {
    #[default]
    #[serde(rename = "text")]
    Text,

    #[serde(rename = "json_schema")]
    JsonSchema(TextFormatJsonSchema),
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct TextFormatJsonSchema {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub schema: Schema,

    pub strict: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum Tool {
    #[serde(rename = "function")]
    Function(ToolFunction),
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolFunction {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub parameters: Schema,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Copy, Clone, Debug, Serialize)]
pub enum ToolChoice {
    #[serde(rename = "none")]
    None,

    #[serde(rename = "auto")]
    Auto,

    #[serde(rename = "required")]
    Required,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Role {
    #[serde(rename = "user")]
    User,

    #[serde(rename = "assistant")]
    Assistant,

    #[serde(rename = "system")]
    System,

    #[serde(rename = "developer")]
    Developer,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum Input {
    #[serde(rename = "message")]
    Message(InputMessage),
    
    #[serde(skip)]
    Template(TemplateInput),
}

#[derive(Clone, Debug)]
pub struct TemplateInput {
    pub role: Role,
    pub template: crate::prompt::PromptTemplate,
}

#[derive(Clone, Debug, Serialize)]
pub struct InputMessage {
    pub role: Role,

    pub content: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Output {
    #[serde(rename = "message")]
    Message(OutputMessage),

    #[serde(rename = "function_call")]
    FunctionCall(OutputFunctionCall),
}

#[derive(Clone, Debug, Deserialize)]
pub struct OutputMessage {
    pub role: Role,

    pub content: Vec<OutputMessageContent>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum OutputMessageContent {
    #[serde(rename = "output_text")]
    OutputText(OutputMessageContentOutputText),

    #[serde(rename = "refusal")]
    Refusal(OutputMessageContentRefusal),
}

#[derive(Clone, Debug, Deserialize)]
pub struct OutputMessageContentOutputText {
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OutputMessageContentRefusal {
    pub refusal: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OutputFunctionCall {
    pub arguments: String,

    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Error {
    pub code: String,

    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Response {
    pub error: Option<Error>,

    pub output: Option<Vec<Output>>,
}
