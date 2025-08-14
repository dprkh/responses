//! Template-first design: explicit loading, fast rendering, clean integration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use serde_yaml;
use serde_json;
use crate::error::{Error, Result};
use crate::prompt::i18n::LocaleManager;
use crate::messages::Messages;

/// A compiled template ready for fast rendering with variables.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    frontmatter: Option<TemplateFrontmatter>,
    content: String,
    default_variables: HashMap<String, serde_json::Value>,
    required_variables: Vec<String>,
    locale_manager: Option<LocaleManager>,
    current_locale: String,
}

/// Template metadata from YAML frontmatter.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateFrontmatter {
    #[serde(default)]
    pub variables: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub required_variables: Vec<String>,
    pub i18n_key: Option<String>,
    #[serde(default)]
    pub includes: Vec<String>,
}

impl PromptTemplate {
    /// Load a template from file - explicit and upfront.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| Error::PromptFileRead {
            path: path.display().to_string(),
            source: e,
        })?;

        Self::from_content(&content)
    }

    /// Create template from content string.
    pub fn from_content(content: &str) -> Result<Self> {
        let (frontmatter, template_content) = Self::parse_frontmatter(content)?;
        
        // Extract default variables from frontmatter
        let mut default_variables = HashMap::new();
        let mut required_variables = Vec::new();
        
        if let Some(ref fm) = frontmatter {
            for (key, value) in &fm.variables {
                if let Ok(json_value) = serde_json::to_value(value) {
                    default_variables.insert(key.clone(), json_value);
                }
            }
            required_variables = fm.required_variables.clone();
        }

        Ok(Self {
            frontmatter,
            content: template_content,
            default_variables,
            required_variables,
            locale_manager: None,
            current_locale: "en".to_string(),
        })
    }

    /// Set the locale for i18n (returns new template, doesn't mutate).
    pub fn with_locale(mut self, locale: &str) -> Result<Self> {
        self.current_locale = locale.to_string();
        
        // Initialize locale manager if we have i18n content
        if self.frontmatter.as_ref().and_then(|f| f.i18n_key.as_ref()).is_some() ||
           self.content.contains("{{i18n.") || self.content.contains("{{t.") {
            // Use a default locales path since this is for basic i18n functionality
            // In a real app, this would come from the app configuration
            if let Ok(manager) = LocaleManager::new("locales", "en") {
                self.locale_manager = Some(manager);
            }
        }
        
        Ok(self)
    }

    /// Get the list of required variables that must be provided at render time.
    pub fn required_variables(&self) -> &[String] {
        &self.required_variables
    }

    /// Validate that all required variables are provided.
    pub fn validate_variables(&self, vars: &serde_json::Value) -> Result<()> {
        if self.required_variables.is_empty() {
            return Ok(());
        }

        let vars_obj = vars.as_object().ok_or_else(|| Error::TemplateParsing(
            "Variables must be provided as a JSON object".to_string()
        ))?;

        let missing: Vec<String> = self.required_variables
            .iter()
            .filter(|&var| !vars_obj.contains_key(var))
            .cloned()
            .collect();

        if !missing.is_empty() {
            return Err(Error::RequiredVariablesMissing { variables: missing });
        }

        Ok(())
    }

    /// Render the template with variables - pure function, fast execution.
    pub fn render(&self, vars: &serde_json::Value) -> Result<String> {
        // Validate required variables
        self.validate_variables(vars)?;

        // Merge default variables with provided variables
        let mut all_vars = self.default_variables.clone();
        if let Some(vars_obj) = vars.as_object() {
            for (key, value) in vars_obj {
                all_vars.insert(key.clone(), value.clone());
            }
        }

        // Render template
        let mut result = self.content.clone();
        
        // 1. Simple variable substitution
        result = self.substitute_variables(&result, &all_vars)?;
        
        // 2. i18n substitution (if needed)
        if self.frontmatter.as_ref().and_then(|f| f.i18n_key.as_ref()).is_some() {
            result = self.substitute_i18n(&result, &all_vars)?;
        }
        
        Ok(result)
    }

    /// Render with a serializable context object - convenient wrapper.
    pub fn render_with_context<T: Serialize>(&self, context: &T) -> Result<String> {
        let vars = serde_json::to_value(context)
            .map_err(|e| Error::TemplateParsing(format!("Failed to serialize context: {}", e)))?;
        self.render(&vars)
    }

    fn parse_frontmatter(content: &str) -> Result<(Option<TemplateFrontmatter>, String)> {
        if !content.starts_with("---") {
            return Ok((None, content.to_string()));
        }

        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Ok((None, content.to_string()));
        }

        let yaml_content = parts[1].trim();
        let template_content = parts[2].to_string();

        if yaml_content.is_empty() {
            return Ok((None, template_content));
        }

        let frontmatter: TemplateFrontmatter = serde_yaml::from_str(yaml_content)
            .map_err(|e| Error::TemplateParsing(format!("Invalid YAML frontmatter: {}", e)))?;

        Ok((Some(frontmatter), template_content))
    }

    fn substitute_variables(&self, content: &str, vars: &HashMap<String, serde_json::Value>) -> Result<String> {
        let mut result = content.to_string();
        
        // Simple {{variable}} substitution
        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "".to_string(),
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                    // For complex objects, we'll handle nested access separately
                    continue;
                }
            };
            result = result.replace(&placeholder, &value_str);
        }

        // Handle nested object access like {{user.name}}
        result = self.substitute_nested_variables(&result, vars)?;

        Ok(result)
    }

    fn substitute_nested_variables(&self, content: &str, vars: &HashMap<String, serde_json::Value>) -> Result<String> {
        let mut result = content.to_string();
        
        // Find patterns like {{object.property}}
        let re = regex::Regex::new(r"\{\{([^}]+\.[^}]+)\}\}")
            .map_err(|e| Error::TemplateParsing(format!("Regex error: {}", e)))?;
            
        for captures in re.captures_iter(content) {
            if let Some(full_match) = captures.get(0) {
                if let Some(key_path) = captures.get(1) {
                    let key_parts: Vec<&str> = key_path.as_str().split('.').collect();
                    if let Some(value) = self.resolve_nested_value(&key_parts, vars) {
                        let value_str = match value {
                            serde_json::Value::String(s) => s,
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            _ => continue,
                        };
                        result = result.replace(full_match.as_str(), &value_str);
                    }
                }
            }
        }
        
        Ok(result)
    }

    fn resolve_nested_value(&self, key_parts: &[&str], vars: &HashMap<String, serde_json::Value>) -> Option<serde_json::Value> {
        if key_parts.is_empty() {
            return None;
        }

        let root_key = key_parts[0];
        let mut current_value = vars.get(root_key)?.clone();

        for &key in &key_parts[1..] {
            match current_value {
                serde_json::Value::Object(ref obj) => {
                    current_value = obj.get(key)?.clone();
                }
                _ => return None,
            }
        }

        Some(current_value)
    }

    fn substitute_i18n(&self, content: &str, _vars: &HashMap<String, serde_json::Value>) -> Result<String> {
        // If we have a locale manager, use it for i18n substitution
        if let Some(ref mut locale_manager) = self.locale_manager.as_ref().cloned() {
            let mut result = content.to_string();
            
            // Find patterns like {{i18n.key}} or {{t.key}}
            let re = regex::Regex::new(r"\{\{(?:i18n|t)\.([^}]+)\}\}")
                .map_err(|e| Error::TemplateParsing(format!("Regex error: {}", e)))?;
                
            for captures in re.captures_iter(content) {
                if let Some(full_match) = captures.get(0) {
                    if let Some(key) = captures.get(1) {
                        // Get locale data and then the string
                        if let Ok(locale_data) = locale_manager.get_locale(&self.current_locale) {
                            if let Some(translated) = locale_data.get_string(key.as_str()) {
                                result = result.replace(full_match.as_str(), &translated);
                            }
                        }
                    }
                }
            }
            
            Ok(result)
        } else {
            // No i18n manager, just return content unchanged
            Ok(content.to_string())
        }
    }
}

/// Application-scale template management.
#[derive(Debug)]
pub struct TemplateSet {
    templates: HashMap<String, PromptTemplate>,
    conversations: HashMap<String, ConversationTemplate>,
    base_path: PathBuf,
    current_locale: String,
}

impl TemplateSet {
    /// Load all templates from a directory.
    pub fn from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let base_path = dir.as_ref().to_path_buf();
        let mut templates = HashMap::new();
        let mut conversations = HashMap::new();

        // Load regular templates
        if let Ok(entries) = fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        let template = PromptTemplate::load(&path)?;
                        templates.insert(name.to_string(), template);
                    }
                }
            }
        }

        // Load conversation templates from conversations/ subdirectory
        let conversations_dir = base_path.join("conversations");
        if conversations_dir.exists() {
            if let Ok(entries) = fs::read_dir(&conversations_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            let conversation = ConversationTemplate::load(&path)?;
                            conversations.insert(name.to_string(), conversation);
                        }
                    }
                }
            }
        }

        Ok(Self {
            templates,
            conversations,
            base_path,
            current_locale: "en".to_string(),
        })
    }

    /// Set the locale for all templates in this set.
    pub fn with_locale(mut self, locale: &str) -> Result<Self> {
        self.current_locale = locale.to_string();
        
        // Apply locale to all templates
        for (_, template) in &mut self.templates {
            *template = template.clone().with_locale(locale)?;
        }
        
        for (_, conversation) in &mut self.conversations {
            *conversation = conversation.clone().with_locale(locale)?;
        }
        
        Ok(self)
    }

    /// Render a template by name.
    pub fn render(&self, template_name: &str, vars: &serde_json::Value) -> Result<String> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| Error::TemplateParsing(format!("Template not found: {}", template_name)))?;
        template.render(vars)
    }

    /// Render a conversation template as Messages.
    pub fn render_conversation(&self, name: &str, vars: &serde_json::Value) -> Result<Messages> {
        let conversation = self.conversations.get(name)
            .ok_or_else(|| Error::TemplateParsing(format!("Conversation template not found: {}", name)))?;
        conversation.render(vars)
    }

    /// List all available template names.
    pub fn list_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a template exists.
    pub fn template_exists(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }

    /// List all available conversation template names.
    pub fn list_conversations(&self) -> Vec<&str> {
        self.conversations.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a conversation template exists.
    pub fn conversation_exists(&self, name: &str) -> bool {
        self.conversations.contains_key(name)
    }

    /// Get current locale.
    pub fn current_locale(&self) -> &str {
        &self.current_locale
    }
}

/// A conversation template that renders to a Messages object.
#[derive(Debug, Clone)]
pub struct ConversationTemplate {
    template: PromptTemplate,
}

impl ConversationTemplate {
    /// Load a conversation template from file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let template = PromptTemplate::load(path)?;
        Ok(Self { template })
    }

    /// Create a conversation template from content string.
    pub fn load_from_content(content: &str) -> Result<Self> {
        let template = PromptTemplate::from_content(content)?;
        Ok(Self { template })
    }

    /// Set the locale for this conversation template.
    pub fn with_locale(mut self, locale: &str) -> Result<Self> {
        self.template = self.template.with_locale(locale)?;
        Ok(self)
    }

    /// Render the conversation template to Messages.
    pub fn render(&self, vars: &serde_json::Value) -> Result<Messages> {
        let content = self.template.render(vars)?;
        
        // Parse conversation sections (## System, ## User, ## Assistant)
        self.parse_conversation_content(&content)
    }

    fn parse_conversation_content(&self, content: &str) -> Result<Messages> {
        let mut messages = Messages::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut current_role = None;
        let mut current_content = Vec::new();
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.starts_with("## ") {
                // Save previous message if any
                if let Some(role) = current_role {
                    let content_str = current_content.join("\n").trim().to_string();
                    if !content_str.is_empty() {
                        messages = messages.add_message(role, content_str);
                    }
                }
                
                // Start new message
                current_content.clear();
                current_role = match trimmed.to_lowercase().as_str() {
                    "## system" => Some(crate::types::Role::System),
                    "## user" => Some(crate::types::Role::User),
                    "## assistant" => Some(crate::types::Role::Assistant),
                    "## developer" => Some(crate::types::Role::Developer),
                    _ => None,
                };
            } else if current_role.is_some() {
                current_content.push(line);
            } else if !trimmed.is_empty() {
                // No section header found, treat as system message
                messages = messages.system(content);
                break;
            }
        }
        
        // Save final message if any
        if let Some(role) = current_role {
            let content_str = current_content.join("\n").trim().to_string();
            if !content_str.is_empty() {
                messages = messages.add_message(role, content_str);
            }
        }
        
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_template() {
        let content = r#"---
variables:
  role: "assistant"
required_variables:
  - "name"
---

Hello {{name}}, I am your {{role}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        
        // Test required variables validation
        assert_eq!(template.required_variables(), &["name"]);
        
        let vars = json!({ "name": "Alice" });
        let result = template.render(&vars).unwrap();
        
        assert_eq!(result, "\n\nHello Alice, I am your assistant.");
    }

    #[test]
    fn test_missing_required_variable() {
        let content = r#"---
required_variables:
  - "name"
---

Hello {{name}}!"#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({});
        
        let result = template.render(&vars);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            Error::RequiredVariablesMissing { variables } => {
                assert_eq!(variables, vec!["name"]);
            }
            _ => panic!("Expected RequiredVariablesMissing error"),
        }
    }

    #[test]
    fn test_nested_variables() {
        let content = "Contact {{user.name}} at {{user.email}}";
        let template = PromptTemplate::from_content(content).unwrap();
        
        let vars = json!({
            "user": {
                "name": "Alice",
                "email": "alice@example.com"
            }
        });
        
        let result = template.render(&vars).unwrap();
        assert_eq!(result, "Contact Alice at alice@example.com");
    }

    #[test]
    fn test_template_without_frontmatter() {
        let content = "Simple template: {{message}}";
        let template = PromptTemplate::from_content(content).unwrap();
        
        let vars = json!({ "message": "Hello World" });
        let result = template.render(&vars).unwrap();
        
        assert_eq!(result, "Simple template: Hello World");
    }

    #[test]
    fn test_render_with_context() {
        #[derive(Serialize)]
        struct Context {
            name: String,
            age: u32,
            active: bool,
        }

        let content = "{{name}} is {{age}} years old and is {{active}}.";
        let template = PromptTemplate::from_content(content).unwrap();
        
        let context = Context {
            name: "Bob".to_string(),
            age: 25,
            active: true,
        };
        
        let result = template.render_with_context(&context).unwrap();
        assert_eq!(result, "Bob is 25 years old and is true.");
    }

    #[test]
    fn test_conversation_template_multi_turn() {
        let content = r#"---
variables:
  topic: "{{topic}}"
  level: "{{level}}"
---

## System
You are teaching {{topic}} to {{level}} students.

## User  
How do I get started with {{topic}}?

## Assistant
Great question! Let me explain {{topic}} basics for {{level}} learners."#;

        let conversation = ConversationTemplate::load_from_content(content).unwrap();
        
        let vars = json!({
            "topic": "Rust programming",
            "level": "beginner"
        });
        
        let messages = conversation.render(&vars).unwrap();
        assert_eq!(messages.len(), 3);
        
        // Check system message
        if let crate::types::Input::Message(msg) = &messages.inputs()[0] {
            assert_eq!(msg.role, crate::types::Role::System);
            assert!(msg.content.contains("teaching Rust programming"));
            assert!(msg.content.contains("beginner students"));
        }
        
        // Check user message
        if let crate::types::Input::Message(msg) = &messages.inputs()[1] {
            assert_eq!(msg.role, crate::types::Role::User);
            assert!(msg.content.contains("get started with Rust programming"));
        }
        
        // Check assistant message
        if let crate::types::Input::Message(msg) = &messages.inputs()[2] {
            assert_eq!(msg.role, crate::types::Role::Assistant);
            assert!(msg.content.contains("Rust programming basics"));
            assert!(msg.content.contains("beginner learners"));
        }
    }

    #[test]
    fn test_conversation_template_single_message() {
        let content = "You are a helpful assistant specializing in {{domain}}.";
        let conversation = ConversationTemplate::load_from_content(content).unwrap();
        
        let vars = json!({ "domain": "mathematics" });
        let messages = conversation.render(&vars).unwrap();
        
        assert_eq!(messages.len(), 1);
        if let crate::types::Input::Message(msg) = &messages.inputs()[0] {
            assert_eq!(msg.role, crate::types::Role::System);
            assert!(msg.content.contains("mathematics"));
        }
    }

    #[test]
    fn test_template_set_from_nonexistent_dir() {
        let result = TemplateSet::from_dir("nonexistent_directory");
        assert!(result.is_ok()); // Should handle missing directory gracefully
        
        let template_set = result.unwrap();
        assert_eq!(template_set.list_templates().len(), 0);
        assert_eq!(template_set.list_conversations().len(), 0);
        assert_eq!(template_set.current_locale(), "en");
    }

    #[test]
    fn test_template_set_locale_switching() {
        let template_set = TemplateSet::from_dir("nonexistent").unwrap();
        let localized_set = template_set.with_locale("es").unwrap();
        assert_eq!(localized_set.current_locale(), "es");
    }

    #[test]
    fn test_i18n_template_initialization() {
        let content = "Hello {{i18n.greeting}} from {{name}}!";
        let template = PromptTemplate::from_content(content).unwrap();
        
        // Locale manager initialization depends on locales directory existing
        // For this test, we just verify that with_locale doesn't crash
        let localized_template = template.with_locale("es").unwrap();
        assert_eq!(localized_template.current_locale, "es");
        
        // In a real application with proper locales directory, locale_manager would be Some
        // For this test, it's ok if it's None since the directory doesn't exist
    }
}