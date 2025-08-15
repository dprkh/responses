//! Prompt template system with internationalization support.
//!
//! This module provides a comprehensive system for managing prompts in external
//! Markdown files with support for variables, internationalization, and template composition.

pub mod i18n;
pub mod template;
pub mod conversation;

pub use template::{PromptTemplate, TemplateSet};
pub use conversation::ConversationTemplate;
pub use i18n::{LocaleManager, LocaleData};