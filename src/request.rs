use crate::{
    error::Result,
    messages::Messages,
    provider::Provider,
    response::Response,
    types::{Input, InputMessage, Role, Tool, ToolChoice},
    Options,
};
use schemars::JsonSchema;
use serde::Deserialize;

pub struct TextRequestBuilder<'a, P: Provider> {
    client: &'a crate::Client<P>,
    options: Options,
}

impl<'a, P: Provider> TextRequestBuilder<'a, P> {
    pub(crate) fn new(client: &'a crate::Client<P>) -> Self {
        Self {
            client,
            options: Options::default(),
        }
    }
    
    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.options.model = Some(model.into());
        self
    }
    
    /// Add a system message. Supports multiline strings seamlessly.
    pub fn system<S: Into<String>>(self, content: S) -> Self {
        let message = InputMessage {
            role: Role::System,
            content: content.into(),
        };
        self.add_message(message)
    }
    
    /// Add a user message. Supports multiline strings seamlessly.
    pub fn user<S: Into<String>>(self, content: S) -> Self {
        let message = InputMessage {
            role: Role::User,
            content: content.into(),
        };
        self.add_message(message)
    }
    
    /// Add an assistant message. Supports multiline strings seamlessly.
    pub fn assistant<S: Into<String>>(self, content: S) -> Self {
        let message = InputMessage {
            role: Role::Assistant,
            content: content.into(),
        };
        self.add_message(message)
    }
    
    /// Add a developer message. Supports multiline strings seamlessly.
    pub fn developer<S: Into<String>>(self, content: S) -> Self {
        let message = InputMessage {
            role: Role::Developer,
            content: content.into(),
        };
        self.add_message(message)
    }
    
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.options.tools = Some(tools);
        self
    }
    
    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.options.tool_choice = Some(choice);
        self
    }
    
    pub fn safety_identifier<S: Into<String>>(mut self, id: S) -> Self {
        self.options.safety_identifier = Some(id.into());
        self
    }
    
    /// Use the provided messages as the conversation history.
    /// This replaces any existing messages.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.options.input = Some(messages.into_inputs());
        self
    }
    
    /// Continue a conversation from existing messages without consuming them.
    /// This replaces any existing messages and allows further chaining.
    pub fn continue_conversation(mut self, messages: &Messages) -> Self {
        self.options.input = Some(messages.inputs().to_vec());
        self
    }
    
    /// Initialize from raw InputMessage vector.
    pub fn from_messages(mut self, messages: Vec<InputMessage>) -> Self {
        let inputs = messages.into_iter()
            .map(Input::Message)
            .collect();
        self.options.input = Some(inputs);
        self
    }
    
    /// Add multiple messages at once using (Role, content) tuples.
    pub fn add_messages<I, S>(mut self, messages: I) -> Self 
    where
        I: IntoIterator<Item = (Role, S)>,
        S: Into<String>,
    {
        for (role, content) in messages {
            self = self.add_message(InputMessage {
                role,
                content: content.into(),
            });
        }
        self
    }
    
    fn add_message(mut self, message: InputMessage) -> Self {
        if self.options.input.is_none() {
            self.options.input = Some(Vec::new());
        }
        
        self.options.input
            .as_mut()
            .unwrap()
            .push(Input::Message(message));
        
        self
    }
    
    /// Add a system message from a template file with variables.
    pub fn system_from_template<PathType: AsRef<std::path::Path>>(self, path: PathType, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.system(content))
    }

    /// Add a user message from a template file with variables.
    pub fn user_from_template<PathType: AsRef<std::path::Path>>(self, path: PathType, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.user(content))
    }

    /// Add an assistant message from a template file with variables.
    pub fn assistant_from_template<PathType: AsRef<std::path::Path>>(self, path: PathType, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.assistant(content))
    }

    /// Add a developer message from a template file with variables.
    pub fn developer_from_template<PathType: AsRef<std::path::Path>>(self, path: PathType, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.developer(content))
    }

    pub async fn send(self) -> Result<Response<String>> {
        self.client.text_with_options(self.options).await
    }
}

pub struct StructuredRequestBuilder<'a, P: Provider, T> {
    client: &'a crate::Client<P>,
    name: String,
    options: Options,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, P: Provider, T> StructuredRequestBuilder<'a, P, T> 
where
    T: JsonSchema + for<'de> Deserialize<'de>,
{
    pub(crate) fn new(client: &'a crate::Client<P>, name: String) -> Self {
        Self {
            client,
            name,
            options: Options::default(),
            _phantom: std::marker::PhantomData,
        }
    }
    
    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.options.model = Some(model.into());
        self
    }
    
    /// Add a system message. Supports multiline strings seamlessly.
    pub fn system<S: Into<String>>(self, content: S) -> Self {
        let message = InputMessage {
            role: Role::System,
            content: content.into(),
        };
        self.add_message(message)
    }
    
    /// Add a user message. Supports multiline strings seamlessly.
    pub fn user<S: Into<String>>(self, content: S) -> Self {
        let message = InputMessage {
            role: Role::User,
            content: content.into(),
        };
        self.add_message(message)
    }
    
    /// Add an assistant message. Supports multiline strings seamlessly.
    pub fn assistant<S: Into<String>>(self, content: S) -> Self {
        let message = InputMessage {
            role: Role::Assistant,
            content: content.into(),
        };
        self.add_message(message)
    }
    
    /// Add a developer message. Supports multiline strings seamlessly.
    pub fn developer<S: Into<String>>(self, content: S) -> Self {
        let message = InputMessage {
            role: Role::Developer,
            content: content.into(),
        };
        self.add_message(message)
    }
    
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.options.tools = Some(tools);
        self
    }
    
    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.options.tool_choice = Some(choice);
        self
    }
    
    pub fn safety_identifier<S: Into<String>>(mut self, id: S) -> Self {
        self.options.safety_identifier = Some(id.into());
        self
    }
    
    /// Use the provided messages as the conversation history.
    /// This replaces any existing messages.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.options.input = Some(messages.into_inputs());
        self
    }
    
    /// Continue a conversation from existing messages without consuming them.
    /// This replaces any existing messages and allows further chaining.
    pub fn continue_conversation(mut self, messages: &Messages) -> Self {
        self.options.input = Some(messages.inputs().to_vec());
        self
    }
    
    /// Initialize from raw InputMessage vector.
    pub fn from_messages(mut self, messages: Vec<InputMessage>) -> Self {
        let inputs = messages.into_iter()
            .map(Input::Message)
            .collect();
        self.options.input = Some(inputs);
        self
    }
    
    /// Add multiple messages at once using (Role, content) tuples.
    pub fn add_messages<I, S>(mut self, messages: I) -> Self 
    where
        I: IntoIterator<Item = (Role, S)>,
        S: Into<String>,
    {
        for (role, content) in messages {
            self = self.add_message(InputMessage {
                role,
                content: content.into(),
            });
        }
        self
    }
    
    fn add_message(mut self, message: InputMessage) -> Self {
        if self.options.input.is_none() {
            self.options.input = Some(Vec::new());
        }
        
        self.options.input
            .as_mut()
            .unwrap()
            .push(Input::Message(message));
        
        self
    }
    
    /// Add a system message from a template file with variables.
    pub fn system_from_template<PathType: AsRef<std::path::Path>>(self, path: PathType, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.system(content))
    }

    /// Add a user message from a template file with variables.
    pub fn user_from_template<PathType: AsRef<std::path::Path>>(self, path: PathType, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.user(content))
    }

    /// Add an assistant message from a template file with variables.
    pub fn assistant_from_template<PathType: AsRef<std::path::Path>>(self, path: PathType, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.assistant(content))
    }

    /// Add a developer message from a template file with variables.
    pub fn developer_from_template<PathType: AsRef<std::path::Path>>(self, path: PathType, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.developer(content))
    }

    pub async fn send(self) -> Result<Response<T>> {
        self.client.structure_with_name_and_options(self.name, self.options).await
    }
}

