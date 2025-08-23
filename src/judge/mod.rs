//! LLM-as-a-Judge functionality for automated evaluation of LLM outputs.
//!
//! This module provides a comprehensive system for using one LLM to evaluate the outputs
//! of another LLM. It supports custom prompts, template-based evaluation criteria, and
//! internationalization through the existing template system.

pub mod assertions;

use crate::client::Client;
use crate::prompt::{PromptTemplate, TemplateSet};
use crate::messages::Messages;
use crate::types::{Role, Input};
use crate::response::Response;
use crate::provider::Provider;
use crate::error::{Error, Result};
use serde::Deserialize;
use schemars::JsonSchema;
use std::path::{Path, PathBuf};


/// Result of an LLM-as-a-judge evaluation
#[derive(Clone, Debug, JsonSchema, Deserialize)]
pub struct Judgment {
    /// Whether the actual behavior matches the expected behavior
    pub passes: bool,
    /// Detailed reasoning for the judgment
    pub reasoning: String,
    /// Optional confidence score (0.0 to 1.0)
    pub confidence: Option<f64>,
}

impl Judgment {
    /// Assert this judgment passes
    pub fn assert_passes(self) -> Self {
        assert!(self.passes, "Judgment failed: {}", self.reasoning);
        self
    }
    
    /// Assert this judgment fails  
    pub fn assert_fails(self) -> Self {
        assert!(!self.passes, "Expected failure but passed: {}", self.reasoning);
        self
    }
    
    /// Assert minimum confidence level
    pub fn assert_confidence(self, min: f64) -> Self {
        if let Some(confidence) = self.confidence {
            assert!(confidence >= min, "Confidence {} below {}: {}", confidence, min, self.reasoning);
        }
        self
    }
}

/// Configuration for judge prompts
pub enum JudgePrompt {
    /// Plain string system prompt (no substitution)
    String(String),
    /// Template from file with variable substitution
    TemplateFile { path: PathBuf, locale: Option<String> },
    /// Template from TemplateSet (for i18n)
    TemplateSet { 
        template_set: TemplateSet, 
        template_name: String, 
        locale: String 
    },
}

/// LLM-as-a-judge evaluator
pub struct Judge<P: Provider> {
    client: Client<P>,
    model: String,
    prompt: Option<JudgePrompt>,
    temperature: Option<f32>,
}

impl<P: Provider> Judge<P> {
    /// Create a new judge instance
    /// 
    /// Note: You must configure a prompt using one of the `with_*` methods before evaluation:
    /// - `.with_prompt(prompt)` for string prompts
    /// - `.with_template_file(path)` for template files  
    /// - `.with_template_directory(path, name)` for template sets
    pub fn new(client: Client<P>, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
            prompt: None, // Must be configured before use
            temperature: Some(0.1), // Low temp for consistent judgments
        }
    }
    
    /// Use custom system prompt (plain string, no substitution)
    pub fn with_prompt(self, prompt: impl Into<String>) -> Self {
        Self {
            prompt: Some(JudgePrompt::String(prompt.into())),
            ..self
        }
    }
    
    /// Use template from file (supports variable substitution)
    pub fn with_template_file<T: AsRef<Path>>(self, path: T) -> Result<Self> {
        Ok(Self {
            prompt: Some(JudgePrompt::TemplateFile { 
                path: path.as_ref().to_path_buf(), 
                locale: None 
            }),
            ..self
        })
    }
    
    /// Use template from file with specific locale
    pub fn with_template_file_and_locale<T: AsRef<Path>>(
        self, 
        path: T, 
        locale: impl Into<String>
    ) -> Result<Self> {
        Ok(Self {
            prompt: Some(JudgePrompt::TemplateFile { 
                path: path.as_ref().to_path_buf(), 
                locale: Some(locale.into()) 
            }),
            ..self
        })
    }
    
    /// Use template directory with i18n support
    pub fn with_template_directory<T: AsRef<Path>>(
        self, 
        path: T, 
        template_name: impl Into<String>
    ) -> Result<Self> {
        let template_set = TemplateSet::from_dir(path)?;
        Ok(Self {
            prompt: Some(JudgePrompt::TemplateSet {
                template_set,
                template_name: template_name.into(),
                locale: "en".to_string(), // Default locale
            }),
            ..self
        })
    }
    
    /// Set locale for template directory usage
    pub fn with_locale(mut self, locale: impl Into<String>) -> Result<Self> {
        match &mut self.prompt {
            Some(JudgePrompt::TemplateSet { locale: current_locale, .. }) => {
                let new_locale = locale.into();
                *current_locale = new_locale.clone();
                // Note: TemplateSet locale changes are applied at render time
            }
            Some(JudgePrompt::TemplateFile { locale: file_locale, .. }) => {
                *file_locale = Some(locale.into());
            }
            Some(JudgePrompt::String(_)) => {
                return Err(Error::Config("Cannot set locale for string prompts".to_string()));
            }
            None => {
                return Err(Error::Config("No prompt configured. Call with_template_file() or with_template_directory() first".to_string()));
            }
        }
        Ok(self)
    }
    
    /// Set temperature for judgment consistency
    ///
    /// Lower temperatures (e.g., 0.1) provide more consistent and deterministic judgments,
    /// which is typically desired for evaluation tasks. Higher temperatures increase variability.
    pub fn with_temperature(self, temp: f32) -> Self {
        Self {
            temperature: Some(temp),
            ..self
        }
    }

    /// Evaluate a conversation and response against expected behavior
    pub async fn evaluate(
        &self,
        conversation_history: &Messages,
        actual_response: &Response<String>, 
        expected_behavior: &str
    ) -> Result<Judgment> {
        if self.prompt.is_none() {
            return Err(Error::Config(
                "No prompt configured. Use with_prompt(), with_template_file(), or with_template_directory() before evaluating".to_string()
            ));
        }
        
        let evaluation_messages = self.build_evaluation_messages(
            conversation_history, 
            actual_response, 
            expected_behavior
        ).await?;
        
        let mut request = self.client
            .structured::<Judgment>()
            .model(&self.model)
            .messages(evaluation_messages);
            
        if let Some(temp) = self.temperature {
            request = request.temperature(temp);
        }
        
        let response = request.send().await?;
        
        response.message
            .ok_or_else(|| Error::InvalidResponse("No judgment received".to_string()))?
            .map_err(|refusal| Error::Refusal(refusal.to_string()))
    }
    
    async fn build_evaluation_messages(
        &self,
        conversation_history: &Messages,
        actual_response: &Response<String>,
        expected_behavior: &str
    ) -> Result<Messages> {
        let formatted_response = self.format_response(actual_response);
        let formatted_conversation = self.format_conversation(conversation_history);
        
        match self.prompt.as_ref().unwrap() { // Safe because we checked in evaluate()
            JudgePrompt::String(system_prompt) => {
                Ok(Messages::new()
                    .system(system_prompt)
                    .user(&format!(
                        "## Conversation History:\n{}\n\n## Actual Response:\n{}\n\n## Expected Behavior:\n{}\n\nEvaluate whether the actual response matches the expected behavior.",
                        formatted_conversation,
                        formatted_response,
                        expected_behavior
                    )))
            },
            JudgePrompt::TemplateFile { path, locale } => {
                let mut template = PromptTemplate::load(path)?;
                if let Some(loc) = locale {
                    template = template.with_locale(loc, &["locales"])?;
                }
                
                let rendered = template
                    .var("conversation_history", formatted_conversation)
                    .var("actual_response", formatted_response)
                    .var("expected_behavior", expected_behavior)
                    .render_with_vars()?;
                
                Ok(Messages::new().system(rendered))
            },
            JudgePrompt::TemplateSet { template_set, template_name, locale } => {
                let vars = serde_json::json!({
                    "conversation_history": formatted_conversation,
                    "actual_response": formatted_response,
                    "expected_behavior": expected_behavior,
                    "_locale": locale  // Pass locale as a variable for template use
                });
                
                let rendered = template_set.render(template_name, &vars)?;
                Ok(Messages::new().system(rendered))
            }
        }
    }
    
    fn format_response(&self, response: &Response<String>) -> String {
        let mut formatted = String::new();
        
        if let Some(text) = response.text_message() {
            formatted.push_str("**Text Response:**\n");
            formatted.push_str(text);
            formatted.push('\n');
        }
        
        if response.has_function_calls() {
            formatted.push_str("\n**Function Calls:**\n");
            for call in &response.function_calls {
                formatted.push_str(&format!("- Function: {}\n", call.name));
                formatted.push_str(&format!("- Arguments: {}\n", call.arguments));
            }
        }
        
        if formatted.is_empty() {
            formatted.push_str("(Empty response)");
        }
        
        formatted
    }
    
    fn format_conversation(&self, messages: &Messages) -> String {
        let mut formatted = String::new();
        
        // Use render_inputs() to get resolved templates
        let rendered_inputs = messages.render_inputs();
        for input in &rendered_inputs {
            match input {
                Input::Message(msg) => {
                    formatted.push_str(&format!("**{}:** {}\n\n", 
                        self.format_role(&msg.role), 
                        msg.content
                    ));
                }
                Input::Template(template_input) => {
                    // Templates should be resolved by render_inputs(), but handle gracefully
                    formatted.push_str(&format!("**{}:** [Template not resolved]\n\n", 
                        self.format_role(&template_input.role)
                    ));
                }
            }
        }
        
        formatted
    }
    
    fn format_role(&self, role: &Role) -> String {
        match role {
            Role::System => "System",
            Role::User => "User", 
            Role::Assistant => "Assistant",
            Role::Developer => "Developer",
        }.to_string()
    }
}