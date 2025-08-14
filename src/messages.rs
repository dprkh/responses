use crate::types::{Input, InputMessage, Role};

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
#[derive(Clone, Debug, Default)]
pub struct Messages {
    messages: Vec<Input>,
}

impl Messages {
    /// Create a new empty message history.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
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
    
    /// Add a system message using a multiline string literal for better formatting.
    /// This is equivalent to `system()` but provides clearer intent for multiline content.
    pub fn system_prompt<S: Into<String>>(self, content: S) -> Self {
        self.system(content)
    }
    
    /// Add a user message using a multiline string literal for better formatting.
    /// This is equivalent to `user()` but provides clearer intent for multiline content.
    pub fn user_message<S: Into<String>>(self, content: S) -> Self {
        self.user(content)
    }
    
    /// Add an assistant message using a multiline string literal for better formatting.
    /// This is equivalent to `assistant()` but provides clearer intent for multiline content.
    pub fn assistant_response<S: Into<String>>(self, content: S) -> Self {
        self.assistant(content)
    }
    
    /// Add a developer message using a multiline string literal for better formatting.
    /// This is equivalent to `developer()` but provides clearer intent for multiline content.
    pub fn developer_note<S: Into<String>>(self, content: S) -> Self {
        self.developer(content)
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
        Self { messages: inputs }
    }
    
    /// Create Messages from input messages.
    pub fn from_messages(messages: Vec<InputMessage>) -> Self {
        let inputs = messages.into_iter()
            .map(Input::Message)
            .collect();
        Self { messages: inputs }
    }
    
    /// Convert to raw Input vector (consumes self).
    pub fn into_inputs(self) -> Vec<Input> {
        self.messages
    }
    
    /// Get a reference to the inputs without consuming.
    pub fn inputs(&self) -> &[Input] {
        &self.messages
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
        }
    }
    
    /// Take the first N messages as a new Messages object.
    pub fn take_first(&self, n: usize) -> Messages {
        let end = std::cmp::min(n, self.messages.len());
        Messages {
            messages: self.messages[..end].to_vec(),
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
                let Input::Message(msg) = input;
                msg.role == role
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

    /// Add a system message from a template file with variables.
    pub fn system_from_template<P: AsRef<std::path::Path>>(self, path: P, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.system(content))
    }

    /// Add a user message from a template file with variables.
    pub fn user_from_template<P: AsRef<std::path::Path>>(self, path: P, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.user(content))
    }

    /// Add an assistant message from a template file with variables.
    pub fn assistant_from_template<P: AsRef<std::path::Path>>(self, path: P, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.assistant(content))
    }

    /// Add a developer message from a template file with variables.
    pub fn developer_from_template<P: AsRef<std::path::Path>>(self, path: P, vars: &serde_json::Value) -> crate::error::Result<Self> {
        let template = crate::prompt::PromptTemplate::load(path)?;
        let content = template.render(vars)?;
        Ok(self.developer(content))
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
            .system_prompt(r#"You are a coding assistant.
Help users with programming questions."#)
            .user_message(r#"How do I:
1. Create a vector
2. Add elements
3. Iterate over it"#)
            .assistant_response(r#"Here's how:

```rust
let mut vec = Vec::new();
vec.push(1);
for item in &vec {
    println!("{}", item);
}
```"#);
            
        assert_eq!(conversation.len(), 3);
        
        // All methods should work the same as their base counterparts
        let Input::Message(msg0) = &conversation.inputs()[0];
        assert!(msg0.content.contains("coding assistant"));
        
        let Input::Message(msg1) = &conversation.inputs()[1];
        assert!(msg1.content.contains("Create a vector"));
        
        let Input::Message(msg2) = &conversation.inputs()[2];
        assert!(msg2.content.contains("Vec::new()"));
    }
}