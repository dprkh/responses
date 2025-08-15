//! Internationalization support for prompt templates.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde_yaml;
use serde::Deserialize;
use crate::error::{Error, Result};

/// Manages locale data and provides i18n functionality for templates.
#[derive(Debug, Clone)]
pub struct LocaleManager {
    locales_path: PathBuf,
    default_locale: String,
    cache: HashMap<String, LocaleData>,
}

/// Contains locale-specific data including translations and formatting.
#[derive(Debug, Clone)]
pub struct LocaleData {
    locale: String,
    strings: HashMap<String, serde_yaml::Value>,
    text_direction: TextDirection,
}


/// Text direction for RTL language support.
#[derive(Debug, Clone)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Deserialize)]
struct LocaleFile {
    #[serde(flatten)]
    strings: HashMap<String, serde_yaml::Value>,
}

impl LocaleManager {
    /// Create a new LocaleManager with the specified locales directory and default locale.
    pub fn new<P: AsRef<Path>>(locales_path: P, default_locale: &str) -> Result<Self> {
        let locales_path = locales_path.as_ref().to_path_buf();
        
        if !locales_path.exists() {
            return Err(Error::Config(format!(
                "Locales directory does not exist: {}",
                locales_path.display()
            )));
        }

        Ok(Self {
            locales_path,
            default_locale: default_locale.to_string(),
            cache: HashMap::new(),
        })
    }

    /// Load locale data for the specified locale, with fallback to parent locales.
    pub fn load_locale(&mut self, locale: &str) -> Result<&LocaleData> {
        if self.cache.contains_key(locale) {
            return Ok(self.cache.get(locale).unwrap());
        }

        let locale_data = self.load_locale_data(locale)?;
        self.cache.insert(locale.to_string(), locale_data);
        Ok(self.cache.get(locale).unwrap())
    }

    /// Resolve locale with fallback chain: es-MX -> es -> default_locale
    pub fn resolve_locale(&mut self, requested_locale: &str) -> Result<String> {
        let fallback_chain = vec![
            requested_locale.to_string(),
            Self::get_parent_locale(requested_locale).unwrap_or_default(),
            self.default_locale.clone(),
        ];

        for locale in fallback_chain {
            if !locale.is_empty() && self.resolve_locale_path(&locale).is_ok() {
                return Ok(locale);
            }
        }

        Err(Error::LocaleNotFound {
            locale: requested_locale.to_string(),
        })
    }

    /// Get locale data for the resolved locale
    pub fn get_locale(&mut self, locale: &str) -> Result<&LocaleData> {
        self.load_locale(locale)
    }

    /// Resolve the filesystem path for a locale.
    pub fn resolve_locale_path(&self, locale: &str) -> Result<PathBuf> {
        let candidates = vec![
            self.locales_path.join(locale),
            self.locales_path.join(Self::get_parent_locale(locale).unwrap_or_default()),
            self.locales_path.join(&self.default_locale),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        Err(Error::LocaleNotFound {
            locale: locale.to_string(),
        })
    }

    /// Check if a locale string is valid (basic format validation).
    pub fn is_valid_locale(&self, locale: &str) -> bool {
        if locale.is_empty() {
            return false;
        }

        // Basic validation: should contain only alphanumeric chars, hyphens, and underscores
        locale.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && !locale.ends_with('-')
            && !locale.ends_with('_')
    }

    /// Get the number of cached locales (for testing).
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    fn load_locale_data(&self, locale: &str) -> Result<LocaleData> {
        let locale_dir = self.resolve_locale_path(locale)?;
        
        // Load all YAML files in the locale directory
        let mut all_strings = HashMap::new();
        
        if locale_dir.is_dir() {
            for entry in fs::read_dir(&locale_dir).map_err(|e| Error::PromptFileRead {
                path: locale_dir.display().to_string(),
                source: e,
            })? {
                let entry = entry.map_err(|e| Error::PromptFileRead {
                    path: locale_dir.display().to_string(),
                    source: e,
                })?;
                
                if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") ||
                   entry.path().extension().and_then(|s| s.to_str()) == Some("yml") {
                    let content = fs::read_to_string(&entry.path()).map_err(|e| Error::PromptFileRead {
                        path: entry.path().display().to_string(),
                        source: e,
                    })?;
                    
                    let locale_file: LocaleFile = serde_yaml::from_str(&content)
                        .map_err(|e| Error::TemplateParsing(format!(
                            "Failed to parse locale file {}: {}",
                            entry.path().display(),
                            e
                        )))?;
                    
                    all_strings.extend(locale_file.strings);
                }
            }
        }

        let text_direction = Self::get_text_direction(locale, &all_strings);

        Ok(LocaleData {
            locale: locale.to_string(),
            strings: all_strings,
            text_direction,
        })
    }

    fn get_parent_locale(locale: &str) -> Option<String> {
        locale.split('-').next().map(|s| s.to_string())
    }


    fn get_text_direction(locale: &str, strings: &HashMap<String, serde_yaml::Value>) -> TextDirection {
        // Check if text direction is explicitly set in locale data
        if let Some(direction) = strings.get("text_direction") {
            if let Some(dir_str) = direction.as_str() {
                if dir_str == "rtl" {
                    return TextDirection::RightToLeft;
                }
            }
        }

        // Default based on language
        let language = locale.split('-').next().unwrap_or(locale);
        match language {
            "ar" | "he" | "fa" | "ur" => TextDirection::RightToLeft,
            _ => TextDirection::LeftToRight,
        }
    }
}

impl LocaleData {
    /// Get a translation string by key (supports nested keys like "system.title").
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get_nested_value(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    /// Interpolate variables into a translation string.
    pub fn interpolate(&self, key: &str, variables: &HashMap<String, serde_json::Value>) -> Result<String> {
        let template = self.get_string(key)
            .ok_or_else(|| Error::I18nKeyNotFound {
                key: key.to_string(),
                locale: self.locale.clone(),
            })?;

        let mut result = template;
        for (var_name, var_value) in variables {
            let placeholder = format!("{{{}}}", var_name);
            let value_str = match var_value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => var_value.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }

        Ok(result)
    }


    /// Format a number according to locale conventions.
    pub fn format_number(&self, number: f64) -> String {
        let language = self.locale.split('-').next().unwrap_or(&self.locale);
        
        match language {
            "de" => {
                // German: 1.234,56
                format!("{:.2}", number).replace(".", ",").replace(",", ".").replace(".", ",")
            }
            _ => {
                // Default (English): 1,234.56
                let formatted = format!("{:.2}", number);
                if number >= 1000.0 {
                    // Add thousands separator (simplified)
                    let parts: Vec<&str> = formatted.split('.').collect();
                    let integer_part = parts[0];
                    let decimal_part = parts.get(1).unwrap_or(&"00");
                    
                    let mut result = String::new();
                    for (i, c) in integer_part.chars().rev().enumerate() {
                        if i > 0 && i % 3 == 0 {
                            result.insert(0, ',');
                        }
                        result.insert(0, c);
                    }
                    format!("{}.{}", result, decimal_part)
                } else {
                    formatted
                }
            }
        }
    }

    /// Format a percentage according to locale conventions.
    pub fn format_percentage(&self, value: f64) -> String {
        let percentage = value * 100.0;
        let formatted = self.format_number(percentage);
        let trimmed = formatted.trim_end_matches(".00").trim_end_matches(".0");
        format!("{}%", trimmed)
    }

    /// Get text direction for this locale.
    pub fn text_direction(&self) -> &str {
        match self.text_direction {
            TextDirection::LeftToRight => "ltr",
            TextDirection::RightToLeft => "rtl",
        }
    }

    fn get_nested_value(&self, key: &str) -> Option<&serde_yaml::Value> {
        let keys: Vec<&str> = key.split('.').collect();
        let mut current = None;
        
        // Find the root key
        if let Some(first_key) = keys.first() {
            current = self.strings.get(*first_key);
        }
        
        // Navigate nested keys
        for key in keys.iter().skip(1) {
            if let Some(value) = current {
                if let Some(mapping) = value.as_mapping() {
                    current = mapping.get(&serde_yaml::Value::String(key.to_string()));
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        
        current
    }
}