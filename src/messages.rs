use crate::types::{Input, InputMessage, Role};
use std::collections::HashMap;

/// A builder for constructing and managing conversation histories.
/// 
/// `Messages` allows you to build complex multi-turn conversations,
/// save them for later reuse, and compose them together.
/// 
/// # Examples
/// 
/// ```rust
/// use responses::Messages;
/// 
/// // Build a conversation history
/// let history = Messages::new()
///     .system("You are a helpful assistant")
///     .user("What's 2+2?")
///     .assistant("2+2 equals 4")
///     .user("What about 3+3?");
/// 
/// // Save for reuse
/// let _saved = history.clone();
/// 
/// println!("Conversation has {} messages", history.len());
/// ```
#[derive(Clone, Debug)]
pub struct Messages {
    messages: Vec<Input>,
    // Fluent API support
    accumulated_variables: HashMap<String, serde_json::Value>,
    current_locale: Option<String>,
    locale_paths: Vec<String>,
}

impl Messages {
    /// Create a new empty message history.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            accumulated_variables: HashMap::new(),
            current_locale: None,
            locale_paths: Vec::new(),
        }
    }
    
    /// Add a system message.
    /// Supports multiline strings seamlessly.
    pub fn system<S: Into<String>>(mut self, content: S) -> Self {
        self.messages.push(Input::Message(InputMessage {
            role: Role::System,
            content: content.into(),
        }));
        self
    }
    
    /// Add a user message.
    /// Supports multiline strings seamlessly.
    pub fn user<S: Into<String>>(mut self, content: S) -> Self {
        self.messages.push(Input::Message(InputMessage {
            role: Role::User,
            content: content.into(),
        }));
        self
    }
    
    /// Add an assistant message.
    /// Supports multiline strings seamlessly.
    pub fn assistant<S: Into<String>>(mut self, content: S) -> Self {
        self.messages.push(Input::Message(InputMessage {
            role: Role::Assistant,
            content: content.into(),
        }));
        self
    }
    
    /// Add a developer message.
    /// Supports multiline strings seamlessly.
    pub fn developer<S: Into<String>>(mut self, content: S) -> Self {
        self.messages.push(Input::Message(InputMessage {
            role: Role::Developer,
            content: content.into(),
        }));
        self
    }
    
    /// Add a message with a specific role.
    /// Supports multiline strings seamlessly.
    pub fn add_message<S: Into<String>>(mut self, role: Role, content: S) -> Self {
        self.messages.push(Input::Message(InputMessage {
            role,
            content: content.into(),
        }));
        self
    }
    
    
    /// Add multiple messages at once.
    pub fn add_messages<I, S>(mut self, messages: I) -> Self 
    where
        I: IntoIterator<Item = (Role, S)>,
        S: Into<String>,
    {
        for (role, content) in messages {
            self.messages.push(Input::Message(InputMessage {
                role,
                content: content.into(),
            }));
        }
        self
    }
    
    /// Extend this message history with another one.
    pub fn extend(mut self, other: Messages) -> Self {
        self.messages.extend(other.messages);
        self
    }
    
    /// Extend this message history with raw inputs.
    pub fn extend_inputs(mut self, inputs: Vec<Input>) -> Self {
        self.messages.extend(inputs);
        self
    }
    
    /// Create a Messages from existing inputs.
    pub fn from_inputs(inputs: Vec<Input>) -> Self {
        Self { 
            messages: inputs,
            accumulated_variables: HashMap::new(),
            current_locale: None,
            locale_paths: Vec::new(),
        }
    }
    
    /// Create Messages from input messages.
    pub fn from_messages(messages: Vec<InputMessage>) -> Self {
        let inputs = messages.into_iter()
            .map(Input::Message)
            .collect();
        Self { 
            messages: inputs,
            accumulated_variables: HashMap::new(),
            current_locale: None,
            locale_paths: Vec::new(),
        }
    }
    
    /// Convert to raw Input vector (consumes self).
    /// Templates are rendered using accumulated variables.
    pub fn into_inputs(self) -> Vec<Input> {
        self.render_inputs()
    }
    
    /// Get a reference to the inputs without consuming.
    pub fn inputs(&self) -> &[Input] {
        &self.messages
    }
    
    /// Get rendered inputs, lazily evaluating any templates.
    /// Templates are rendered using accumulated variables.
    pub fn render_inputs(&self) -> Vec<Input> {
        self.messages.iter().map(|input| {
            match input {
                Input::Message(msg) => Input::Message(msg.clone()),
                Input::Template(template_input) => {
                    // Apply accumulated variables and render the template
                    let template_with_vars = Self::apply_accumulated_variables_static(&self.accumulated_variables, template_input.template.clone());
                    let template_with_locale = if let Some(ref locale) = self.current_locale {
                        template_with_vars.clone().with_locale(locale, &self.locale_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>()).unwrap_or(template_with_vars)
                    } else {
                        template_with_vars
                    };
                    
                    match template_with_locale.render_with_vars() {
                        Ok(content) => Input::Message(InputMessage {
                            role: template_input.role.clone(),
                            content,
                        }),
                        Err(_) => Input::Message(InputMessage {
                            role: template_input.role.clone(),
                            content: format!("<!-- Template rendering failed -->"),
                        }),
                    }
                }
            }
        }).collect()
    }
    
    /// Get the number of messages.
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    /// Get the last message, if any.
    pub fn last(&self) -> Option<&Input> {
        self.messages.last()
    }
    
    /// Get the first message, if any.
    pub fn first(&self) -> Option<&Input> {
        self.messages.first()
    }
    
    /// Take the last N messages as a new Messages object.
    pub fn take_last(&self, n: usize) -> Messages {
        let start = if n >= self.messages.len() {
            0
        } else {
            self.messages.len() - n
        };
        
        Messages {
            messages: self.messages[start..].to_vec(),
            accumulated_variables: HashMap::new(),
            current_locale: None,
            locale_paths: Vec::new(),
        }
    }
    
    /// Take the first N messages as a new Messages object.
    pub fn take_first(&self, n: usize) -> Messages {
        let end = std::cmp::min(n, self.messages.len());
        Messages {
            messages: self.messages[..end].to_vec(),
            accumulated_variables: HashMap::new(),
            current_locale: None,
            locale_paths: Vec::new(),
        }
    }
    
    /// Remove the last N messages.
    pub fn pop_last(mut self, n: usize) -> Self {
        let new_len = self.messages.len().saturating_sub(n);
        self.messages.truncate(new_len);
        self
    }
    
    /// Filter messages by role.
    pub fn filter_by_role(&self, role: Role) -> Messages {
        let filtered: Vec<Input> = self.messages
            .iter()
            .filter(|input| {
                match input {
                    Input::Message(msg) => msg.role == role,
                    Input::Template(template_input) => template_input.role == role,
                }
            })
            .cloned()
            .collect();
        
        Messages::from_inputs(filtered)
    }
    
    /// Create a conversation template that can be reused.
    /// This is useful for system prompts and initial context.
    pub fn as_template(&self) -> Messages {
        self.clone()
    }
    
    

    /// Create Messages from a conversation template file.
    pub fn from_conversation_template<P: AsRef<std::path::Path>>(path: P, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let conversation = crate::prompt::ConversationTemplate::load(path)?;
        conversation.render(vars)
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

    /// Set the locale for template rendering (fluent API)
    pub fn with_locale<S: Into<String>>(mut self, locale: S, locale_paths: &[&str]) -> crate::error::Result<Self> {
        self.current_locale = Some(locale.into());
        self.locale_paths = locale_paths.iter().map(|s| s.to_string()).collect();
        Ok(self)
    }

    // Internal helper methods
    fn system_from_template_internal(mut self, template: crate::prompt::PromptTemplate) -> crate::error::Result<Self> {
        // Store template for lazy evaluation instead of rendering immediately
        self.messages.push(crate::types::Input::Template(crate::types::TemplateInput {
            role: crate::types::Role::System,
            template,
        }));
        Ok(self)
    }

    fn assistant_from_template_internal(mut self, template: crate::prompt::PromptTemplate) -> crate::error::Result<Self> {
        // Store template for lazy evaluation instead of rendering immediately
        self.messages.push(crate::types::Input::Template(crate::types::TemplateInput {
            role: crate::types::Role::Assistant,
            template,
        }));
        Ok(self)
    }

    fn apply_accumulated_variables_static(vars: &HashMap<String, serde_json::Value>, mut template: crate::prompt::PromptTemplate) -> crate::prompt::PromptTemplate {
        for (key, value) in vars {
            template = template.var(key, value.clone());
        }
        template
    }
}

impl Default for Messages {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a new Messages builder.
pub fn messages() -> Messages {
    Messages::new()
}

/// Macro for easier message construction.
#[macro_export]
macro_rules! messages {
    () => {
        $crate::messages::Messages::new()
    };
    (
        $(
            $role:ident: $content:expr
        ),* $(,)?
    ) => {{
        let mut msgs = $crate::messages::Messages::new();
        $(
            msgs = msgs.$role($content);
        )*
        msgs
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_messages_builder() {
        let messages = Messages::new()
            .system("You are helpful")
            .user("Hello")
            .assistant("Hi there!")
            .user("How are you?");
            
        assert_eq!(messages.len(), 4);
        assert!(!messages.is_empty());
    }
    
    #[test]
    fn test_messages_macro() {
        let messages = messages! {
            system: "You are helpful",
            user: "Hello",
            assistant: "Hi there!"
        };
        
        assert_eq!(messages.len(), 3);
    }
    
    #[test]
    fn test_conversation_slicing() {
        let conversation = Messages::new()
            .system("System prompt")
            .user("Question 1")
            .assistant("Answer 1")
            .user("Question 2")
            .assistant("Answer 2");
            
        let last_exchange = conversation.take_last(2);
        assert_eq!(last_exchange.len(), 2);
        
        let system_only = conversation.filter_by_role(Role::System);
        assert_eq!(system_only.len(), 1);
    }
    
    #[test]
    fn test_multiline_messages() {
        let multiline_system = r#"You are a helpful assistant.

Your responses should:
- Be clear and concise
- Include examples when helpful
- Ask for clarification when needed"#;

        let multiline_user = r#"I need help with this code:

```rust
fn main() {
    println!("Hello, world!");
}
```

Can you explain what this does?"#;

        let conversation = Messages::new()
            .system(multiline_system)
            .user(multiline_user);
            
        assert_eq!(conversation.len(), 2);
        
        // Verify that multiline content is preserved
        if let Some(Input::Message(msg)) = conversation.inputs().get(0) {
            assert!(msg.content.contains("Your responses should:"));
            assert!(msg.content.contains("- Be clear and concise"));
        }
        
        if let Some(Input::Message(msg)) = conversation.inputs().get(1) {
            assert!(msg.content.contains("```rust"));
            assert!(msg.content.contains("println!"));
        }
    }
    
    #[test]
    fn test_multiline_convenience_methods() {
        let conversation = Messages::new()
            .system(r#"You are a coding assistant.
Help users with programming questions."#)
            .user(r#"How do I:
1. Create a vector
2. Add elements
3. Iterate over it"#)
            .assistant(r#"Here's how:

```rust
let mut vec = Vec::new();
vec.push(1);
for item in &vec {
    println!("{}", item);
}
```"#);
            
        assert_eq!(conversation.len(), 3);
        
        // All methods should work the same as their base counterparts
        let inputs = conversation.render_inputs();
        if let Input::Message(msg0) = &inputs[0] {
            assert!(msg0.content.contains("coding assistant"));
        }
        
        if let Input::Message(msg1) = &inputs[1] {
            assert!(msg1.content.contains("Create a vector"));
        }
        
        if let Input::Message(msg2) = &inputs[2] {
            assert!(msg2.content.contains("Vec::new()"));
        }
    }
}