//! Conversation template support for multi-turn interactions.

use std::path::Path;
use std::collections::HashMap;
use crate::error::Result;
use crate::messages::Messages;
use crate::prompt::template::PromptTemplate;
use crate::types::Role;

/// A conversation template that can generate multi-turn conversations.
pub struct ConversationTemplate {
    template: PromptTemplate,
    accumulated_variables: HashMap<String, serde_json::Value>,
}

impl ConversationTemplate {
    /// Load a conversation template from a file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let template = PromptTemplate::load(path)?;
        Ok(Self { 
            template,
            accumulated_variables: HashMap::new(),
        })
    }

    /// Set a template variable (fluent API)
    pub fn var<K: Into<String>, V: serde::Serialize>(mut self, key: K, value: V) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.accumulated_variables.insert(key.into(), json_value);
        }
        self
    }

    /// Render the conversation template to a Messages object using accumulated variables.
    pub fn to_messages(self) -> Result<Messages> {
        let vars = serde_json::Value::Object(self.accumulated_variables.clone().into_iter().collect());
        self.render(&vars)
    }

    /// Render the conversation template to a Messages object.
    pub fn render(&self, vars: &serde_json::Value) -> Result<Messages> {
        let rendered = self.template.render(vars)?;
        self.parse_conversation(&rendered)
    }

    /// Parse a rendered conversation template into Messages.
    /// 
    /// Conversation templates should use markdown headers to denote different speakers:
    /// ## System
    /// ## User  
    /// ## Assistant
    /// ## Developer
    fn parse_conversation(&self, content: &str) -> Result<Messages> {
        let mut messages = Messages::new();
        let mut current_role: Option<Role> = None;
        let mut current_content = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            
            // Check for role headers
            if let Some(role) = self.parse_role_header(trimmed) {
                // Save previous message if any
                if let Some(prev_role) = current_role {
                    if !current_content.trim().is_empty() {
                        messages = messages.add_message(prev_role, current_content.trim());
                    }
                }
                
                // Start new message
                current_role = Some(role);
                current_content.clear();
            } else if current_role.is_some() {
                // Accumulate content for current role
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
            }
            // Skip content before any role header
        }

        // Save final message
        if let Some(role) = current_role {
            if !current_content.trim().is_empty() {
                messages = messages.add_message(role, current_content.trim());
            }
        }

        // If no role headers found, treat entire content as system message
        if messages.is_empty() && !content.trim().is_empty() {
            messages = messages.system(content.trim());
        }

        Ok(messages)
    }

    /// Parse role headers like "## System", "## User", etc.
    fn parse_role_header(&self, line: &str) -> Option<Role> {
        if !line.starts_with("## ") {
            return None;
        }

        let role_text = line[3..].trim().to_lowercase();
        match role_text.as_str() {
            "system" => Some(Role::System),
            "user" => Some(Role::User),
            "assistant" => Some(Role::Assistant),
            "developer" => Some(Role::Developer),
            _ => None,
        }
    }
}