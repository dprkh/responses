//! Conversation template support for multi-turn interactions.

use std::path::Path;
use crate::error::Result;
use crate::messages::Messages;
use crate::prompt::template::PromptTemplate;

/// A conversation template that can generate multi-turn conversations.
pub struct ConversationTemplate {
    template: PromptTemplate,
}

impl ConversationTemplate {
    /// Load a conversation template from a file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let template = PromptTemplate::load(path)?;
        Ok(Self { template })
    }

    /// Render the conversation template to a Messages object.
    pub fn render(&self, vars: &serde_json::Value) -> Result<Messages> {
        // For now, just render as a single system message
        // Real implementation would parse conversation structure
        let rendered = self.template.render(vars)?;
        Ok(Messages::new().system(rendered))
    }
}