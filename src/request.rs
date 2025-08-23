use crate::{
    error::Result,
    messages::Messages,
    provider::Provider,
    response::Response,
    types::{Input, InputMessage, Role, Tool, ToolChoice},
    Options,
};
use std::collections::HashMap;
use schemars::JsonSchema;
use serde::Deserialize;

pub struct TextRequestBuilder<'a, P: Provider> {
    client: &'a crate::Client<P>,
    options: Options,
    // Fluent API support
    accumulated_variables: HashMap<String, serde_json::Value>,
    current_locale: Option<String>,
    locale_paths: Vec<String>,
}

impl<'a, P: Provider> TextRequestBuilder<'a, P> {
    pub(crate) fn new(client: &'a crate::Client<P>) -> Self {
        Self {
            client,
            options: Options::default(),
            accumulated_variables: HashMap::new(),
            current_locale: None,
            locale_paths: Vec::new(),
        }
    }
    
    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.options.model = Some(model.into());
        self
    }
    
    /// Set model using a static string reference
    /// 
    /// # Example
    /// ```rust,no_run
    /// # use responses::{azure, Client};
    /// # use responses::provider::ProviderBuilder;
    /// # #[tokio::main] async fn main() -> responses::Result<()> {
    /// let provider = azure().from_env()?.build()?;
    /// let client = Client::new(provider);
    /// let response = client.text()
    ///     .with_model("gpt-4o")
    ///     .user("Hello")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_model(self, model: &'static str) -> Self {
        self.model(model)
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
    

    // === FLUENT API METHODS ===
    // Support for builder pattern with template variables

    /// Add a system message from a markdown template file (fluent API)
    pub fn system_from_md<PathType: AsRef<std::path::Path>>(self, path: PathType) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let template = if let Some(ref locale) = self.current_locale {
            template.with_locale(locale, &self.locale_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>())?
        } else {
            template
        };
        self.system_from_template_internal(template)
    }

    /// Add an assistant message from a markdown template file (fluent API)
    pub fn assistant_from_md<PathType: AsRef<std::path::Path>>(self, path: PathType) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let template = if let Some(ref locale) = self.current_locale {
            template.with_locale(locale, &self.locale_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>())?
        } else {
            template
        };
        self.assistant_from_template_internal(template)
    }

    /// Set a template variable (fluent API)
    pub fn var<K: Into<String>, V: serde::Serialize>(mut self, key: K, value: V) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.accumulated_variables.insert(key.into(), json_value);
        }
        self
    }

    /// Set the locale and locale paths for template rendering (fluent API)
    pub fn with_locale<S: Into<String>>(mut self, locale: S, locale_paths: &[&str]) -> crate::error::Result<Self> {
        self.current_locale = Some(locale.into());
        self.locale_paths = locale_paths.iter().map(|s| s.to_string()).collect();
        Ok(self)
    }

    // Internal helper methods
    fn add_template(mut self, template_input: crate::types::TemplateInput) -> Self {
        if self.options.input.is_none() {
            self.options.input = Some(Vec::new());
        }
        
        self.options.input
            .as_mut()
            .unwrap()
            .push(Input::Template(template_input));
        
        self
    }
    
    fn system_from_template_internal(self, template: crate::prompt::PromptTemplate) -> crate::error::Result<Self> {
        Ok(self.add_template(crate::types::TemplateInput {
            role: crate::types::Role::System,
            template,
        }))
    }

    fn assistant_from_template_internal(self, template: crate::prompt::PromptTemplate) -> crate::error::Result<Self> {
        Ok(self.add_template(crate::types::TemplateInput {
            role: crate::types::Role::Assistant,
            template,
        }))
    }

    

    pub async fn send(self) -> Result<Response<String>> {
        let Self { client, options, accumulated_variables, current_locale, locale_paths: _ } = self;
        let mut rendered_options = options;
        
        // Render templates if any exist
        if let Some(ref inputs) = rendered_options.input {
            let mut rendered_inputs = Vec::new();
            
            for input in inputs {
                match input {
                    Input::Message(_msg) => rendered_inputs.push(input.clone()),
                    Input::Template(template_input) => {
                        // Apply accumulated variables and locale to template
                        let mut template = template_input.template.clone();
                        
                        // Apply accumulated variables
                        for (key, value) in &accumulated_variables {
                            template = template.var(key, value.clone());
                        }
                        
                        // Apply current locale if set
                        if let Some(ref locale) = current_locale {
                            template = template.with_locale(locale, &self.locale_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;
                        }
                        
                        // Render template
                        let content = template.render_with_vars()?;
                        
                        // Create message from rendered content
                        rendered_inputs.push(Input::Message(InputMessage {
                            role: template_input.role.clone(),
                            content,
                        }));
                    }
                }
            }
            
            rendered_options.input = Some(rendered_inputs);
        }
        
        client.text_with_options(rendered_options).await
    }
}

pub struct StructuredRequestBuilder<'a, P: Provider, T> {
    client: &'a crate::Client<P>,
    name: String,
    options: Options,
    _phantom: std::marker::PhantomData<T>,
    // Fluent API support
    accumulated_variables: HashMap<String, serde_json::Value>,
    current_locale: Option<String>,
    locale_paths: Vec<String>,
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
            accumulated_variables: HashMap::new(),
            current_locale: None,
            locale_paths: Vec::new(),
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
    

    // === FLUENT API METHODS ===
    // Support for builder pattern with template variables

    /// Add a system message from a markdown template file (fluent API)
    pub fn system_from_md<PathType: AsRef<std::path::Path>>(self, path: PathType) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let template = if let Some(ref locale) = self.current_locale {
            template.with_locale(locale, &self.locale_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>())?
        } else {
            template
        };
        self.system_from_template_internal(template)
    }

    /// Add an assistant message from a markdown template file (fluent API)
    pub fn assistant_from_md<PathType: AsRef<std::path::Path>>(self, path: PathType) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let template = if let Some(ref locale) = self.current_locale {
            template.with_locale(locale, &self.locale_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>())?
        } else {
            template
        };
        self.assistant_from_template_internal(template)
    }

    /// Set a template variable (fluent API)
    pub fn var<K: Into<String>, V: serde::Serialize>(mut self, key: K, value: V) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.accumulated_variables.insert(key.into(), json_value);
        }
        self
    }

    /// Set the locale and locale paths for template rendering (fluent API)
    pub fn with_locale<S: Into<String>>(mut self, locale: S, locale_paths: &[&str]) -> crate::error::Result<Self> {
        self.current_locale = Some(locale.into());
        self.locale_paths = locale_paths.iter().map(|s| s.to_string()).collect();
        Ok(self)
    }

    // Internal helper methods
    fn add_template(mut self, template_input: crate::types::TemplateInput) -> Self {
        if self.options.input.is_none() {
            self.options.input = Some(Vec::new());
        }
        
        self.options.input
            .as_mut()
            .unwrap()
            .push(Input::Template(template_input));
        
        self
    }
    
    fn system_from_template_internal(self, template: crate::prompt::PromptTemplate) -> crate::error::Result<Self> {
        Ok(self.add_template(crate::types::TemplateInput {
            role: crate::types::Role::System,
            template,
        }))
    }

    fn assistant_from_template_internal(self, template: crate::prompt::PromptTemplate) -> crate::error::Result<Self> {
        Ok(self.add_template(crate::types::TemplateInput {
            role: crate::types::Role::Assistant,
            template,
        }))
    }

    

    pub async fn send(self) -> Result<Response<T>> {
        let Self { client, name, options, accumulated_variables, current_locale, .. } = self;
        let mut rendered_options = options;
        
        // Render templates if any exist
        if let Some(ref inputs) = rendered_options.input {
            let mut rendered_inputs = Vec::new();
            
            for input in inputs {
                match input {
                    Input::Message(_msg) => rendered_inputs.push(input.clone()),
                    Input::Template(template_input) => {
                        // Apply accumulated variables and locale to template
                        let mut template = template_input.template.clone();
                        
                        // Apply accumulated variables
                        for (key, value) in &accumulated_variables {
                            template = template.var(key, value.clone());
                        }
                        
                        // Apply current locale if set
                        if let Some(ref locale) = current_locale {
                            template = template.with_locale(locale, &self.locale_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;
                        }
                        
                        // Render template
                        let content = template.render_with_vars()?;
                        
                        // Create message from rendered content
                        rendered_inputs.push(Input::Message(InputMessage {
                            role: template_input.role.clone(),
                            content,
                        }));
                    }
                }
            }
            
            rendered_options.input = Some(rendered_inputs);
        }
        
        client.structure_with_name_and_options(name, rendered_options).await
    }
}

