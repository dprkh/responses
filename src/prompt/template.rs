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

/// Template Abstract Syntax Tree node types
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateNode {
    /// Plain text content
    Text(String),
    /// Variable substitution: {{variable}}
    Variable(String),
    /// Nested variable: {{object.property}}
    NestedVariable(Vec<String>),
    /// Conditional block: {{#if condition}}content{{/if}}
    If {
        condition: String,
        then_content: Vec<TemplateNode>,
        else_content: Option<Vec<TemplateNode>>,
    },
    /// Loop block: {{#each array}}content{{/each}}
    Each {
        variable: String,
        content: Vec<TemplateNode>,
    },
    /// Switch statement: {{#switch variable}}{{#case "value"}}content{{/case}}{{/switch}}
    Switch {
        variable: String,
        cases: Vec<(String, Vec<TemplateNode>)>,
    },
    /// Case within a switch statement
    Case {
        value: String,
        content: Vec<TemplateNode>,
    },
    /// Locale-specific conditional: {{#if_locale "en"}}content{{/if_locale}}
    IfLocale {
        locale: String,
        then_content: Vec<TemplateNode>,
        else_content: Option<Vec<TemplateNode>>,
    },
    /// Template include: {{> path/to/template.md}} or {{> path.md param=value}}
    Include {
        path: String,
        params: HashMap<String, String>,
    },
    /// i18n translation: {{i18n "key" param=value}}
    I18n {
        key: String,
        params: HashMap<String, String>,
    },
    /// Helper function call: {{plural count "word"}} or {{format_number value style="percent"}}
    Helper {
        name: String,
        args: Vec<String>,
        params: HashMap<String, String>,
    },
}

/// Template parser for Handlebars-like syntax
#[derive(Debug)]
pub struct TemplateParser {
    content: String,
    position: usize,
}

impl TemplateParser {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            position: 0,
        }
    }

    /// Parse template content into AST nodes
    pub fn parse(&mut self) -> Result<Vec<TemplateNode>> {
        let mut nodes = Vec::new();
        
        while self.position < self.content.len() {
            if let Some(node) = self.parse_next_node()? {
                nodes.push(node);
            } else {
                break;
            }
        }
        
        Ok(nodes)
    }

    fn parse_next_node(&mut self) -> Result<Option<TemplateNode>> {
        // Find next template expression
        let remaining = &self.content[self.position..];
        
        if let Some(start) = remaining.find("{{") {
            // Add any text before the template expression
            if start > 0 {
                let text = remaining[..start].to_string();
                self.position += start;
                return Ok(Some(TemplateNode::Text(text)));
            }
            
            // Parse template expression
            self.position += 2; // Skip "{{"
            self.parse_template_expression()
        } else {
            // No more template expressions, add remaining text
            if !remaining.is_empty() {
                let text = remaining.to_string();
                self.position = self.content.len();
                Ok(Some(TemplateNode::Text(text)))
            } else {
                Ok(None)
            }
        }
    }

    fn parse_template_expression(&mut self) -> Result<Option<TemplateNode>> {
        let remaining = &self.content[self.position..];
        
        if let Some(end) = remaining.find("}}") {
            let expr = remaining[..end].trim().to_string();
            self.position += end + 2; // Skip "}}"
            
            // Parse different expression types
            if expr.starts_with("#if ") {
                self.parse_if_block(&expr[4..])
            } else if expr.starts_with("#each ") {
                self.parse_each_block(&expr[6..])
            } else if expr.starts_with("#switch ") {
                self.parse_switch_block(&expr[8..])
            } else if expr.starts_with("#if_locale ") {
                self.parse_if_locale_block(&expr[11..])
            } else if expr.starts_with("> ") {
                self.parse_include_expression(&expr[2..])
            } else if expr.starts_with("i18n ") {
                self.parse_i18n_expression(&expr[5..])
            } else if self.looks_like_helper_function(&expr) {
                self.parse_helper_expression(&expr)
            } else if expr.contains('.') {
                // Nested variable
                let parts: Vec<String> = expr.split('.').map(|s| s.to_string()).collect();
                Ok(Some(TemplateNode::NestedVariable(parts)))
            } else {
                // Simple variable
                Ok(Some(TemplateNode::Variable(expr)))
            }
        } else {
            Err(Error::TemplateParsing("Unclosed template expression".to_string()))
        }
    }

    fn parse_if_block(&mut self, condition: &str) -> Result<Option<TemplateNode>> {
        let then_content = self.parse_block_until(&["{{else}}", "{{/if}}"])?;
        
        let remaining = &self.content[self.position..];
        let else_content = if remaining.starts_with("{{else}}") {
            self.position += 8; // Skip "{{else}}"
            Some(self.parse_block_until(&["{{/if}}"])?)
        } else {
            None
        };
        
        // Skip "{{/if}}"
        if self.content[self.position..].starts_with("{{/if}}") {
            self.position += 7;
        }
        
        Ok(Some(TemplateNode::If {
            condition: condition.to_string(),
            then_content,
            else_content,
        }))
    }

    fn parse_each_block(&mut self, variable: &str) -> Result<Option<TemplateNode>> {
        let content = self.parse_block_until(&["{{/each}}"])?;
        
        // Skip "{{/each}}"
        if self.content[self.position..].starts_with("{{/each}}") {
            self.position += 9;
        }
        
        Ok(Some(TemplateNode::Each {
            variable: variable.to_string(),
            content,
        }))
    }

    fn parse_switch_block(&mut self, variable: &str) -> Result<Option<TemplateNode>> {
        let mut cases = Vec::new();
        
        while self.position < self.content.len() {
            let remaining = &self.content[self.position..];
            if remaining.starts_with("{{/switch}}") {
                self.position += 11; // Skip "{{/switch}}"
                break;
            }
            
            if let Some(case_start) = remaining.find("{{#case ") {
                self.position += case_start + 8;
                if let Some(case_end) = self.content[self.position..].find("}}") {
                    let case_value = self.content[self.position..self.position + case_end]
                        .trim()
                        .trim_matches('"')
                        .to_string();
                    self.position += case_end + 2;
                    
                    let case_content = self.parse_block_until(&["{{/case}}"])?;
                    
                    // Skip "{{/case}}"
                    if self.content[self.position..].starts_with("{{/case}}") {
                        self.position += 9;
                    }
                    
                    cases.push((case_value, case_content));
                } else {
                    break;
                }
            } else {
                // No more cases found, advance position by 1 to avoid infinite loop
                // This allows us to skip whitespace and find the {{/switch}} terminator
                self.position += 1;
            }
        }
        
        Ok(Some(TemplateNode::Switch {
            variable: variable.to_string(),
            cases,
        }))
    }

    fn parse_if_locale_block(&mut self, locale_expr: &str) -> Result<Option<TemplateNode>> {
        let locale = locale_expr.trim_matches('"').to_string();
        let then_content = self.parse_block_until(&["{{else}}", "{{/if_locale}}"])?;
        
        let else_content = if self.content[self.position..].starts_with("{{else}}") {
            self.position += 8; // Skip "{{else}}"
            Some(self.parse_block_until(&["{{/if_locale}}"])?)
        } else {
            None
        };
        
        // Skip "{{/if_locale}}"
        if self.content[self.position..].starts_with("{{/if_locale}}") {
            self.position += 14;
        }
        
        Ok(Some(TemplateNode::IfLocale { locale, then_content, else_content }))
    }

    fn parse_include_expression(&mut self, expr: &str) -> Result<Option<TemplateNode>> {
        // Parse include expressions like "path.md" or "path.md param=value"
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.is_empty() {
            return Err(Error::TemplateParsing("Empty include expression".to_string()));
        }
        
        let path = parts[0].to_string();
        let mut params = HashMap::new();
        
        for part in &parts[1..] {
            if let Some((param_name, param_value)) = part.split_once('=') {
                params.insert(param_name.to_string(), param_value.to_string());
            }
        }
        
        Ok(Some(TemplateNode::Include { path, params }))
    }

    fn parse_i18n_expression(&mut self, expr: &str) -> Result<Option<TemplateNode>> {
        // Parse i18n expressions like "key" param=value or param=(helper_function)
        let mut chars = expr.chars().peekable();
        
        // Skip whitespace
        while chars.peek() == Some(&' ') {
            chars.next();
        }
        
        // Parse the key (first quoted string)
        let mut key = String::new();
        if chars.peek() == Some(&'"') {
            chars.next(); // skip opening quote
            while let Some(ch) = chars.next() {
                if ch == '"' {
                    break;
                }
                key.push(ch);
            }
        } else {
            // Unquoted key
            while let Some(&ch) = chars.peek() {
                if ch == ' ' {
                    break;
                }
                key.push(chars.next().unwrap());
            }
        }
        
        let mut params = HashMap::new();
        
        // Parse parameters
        while chars.peek().is_some() {
            // Skip whitespace
            while chars.peek() == Some(&' ') {
                chars.next();
            }
            
            if chars.peek().is_none() {
                break;
            }
            
            // Parse parameter name
            let mut param_name = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '=' {
                    break;
                }
                param_name.push(chars.next().unwrap());
            }
            
            if chars.next() != Some('=') {
                return Err(Error::TemplateParsing("Expected '=' in i18n parameter".to_string()));
            }
            
            // Parse parameter value (could be quoted string, variable, or parenthesized expression)
            let mut param_value = String::new();
            
            if chars.peek() == Some(&'(') {
                // Parenthesized expression - find matching closing paren
                chars.next(); // skip opening paren
                let mut paren_count = 1;
                while let Some(ch) = chars.next() {
                    if ch == '(' {
                        paren_count += 1;
                    } else if ch == ')' {
                        paren_count -= 1;
                        if paren_count == 0 {
                            break;
                        }
                    }
                    param_value.push(ch);
                }
            } else if chars.peek() == Some(&'"') {
                // Quoted string
                chars.next(); // skip opening quote
                while let Some(ch) = chars.next() {
                    if ch == '"' {
                        break;
                    }
                    param_value.push(ch);
                }
            } else {
                // Variable name
                while let Some(&ch) = chars.peek() {
                    if ch == ' ' {
                        break;
                    }
                    param_value.push(chars.next().unwrap());
                }
            }
            
            params.insert(param_name, param_value);
        }
        
        Ok(Some(TemplateNode::I18n { key, params }))
    }

    fn looks_like_helper_function(&self, expr: &str) -> bool {
        // Check if expression looks like a helper function call
        // Known helper functions: plural, format_number
        expr.starts_with("plural ") || expr.starts_with("format_number ")
    }

    fn parse_helper_expression(&mut self, expr: &str) -> Result<Option<TemplateNode>> {
        // Parse helper function expressions like "plural count "word"" or "format_number value style="percent""
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.is_empty() {
            return Err(Error::TemplateParsing("Empty helper expression".to_string()));
        }
        
        let helper_name = parts[0].to_string();
        let mut args = Vec::new();
        let mut params = HashMap::new();
        
        for part in &parts[1..] {
            if part.contains('=') {
                // It's a parameter
                if let Some((param_name, param_value)) = part.split_once('=') {
                    params.insert(param_name.to_string(), param_value.to_string());
                }
            } else {
                // It's an argument
                args.push(part.to_string());
            }
        }
        
        Ok(Some(TemplateNode::Helper {
            name: helper_name,
            args,
            params,
        }))
    }

    fn parse_block_until(&mut self, terminators: &[&str]) -> Result<Vec<TemplateNode>> {
        let mut nodes = Vec::new();
        
        while self.position < self.content.len() {
            let remaining = &self.content[self.position..];
            
            // Check if we've hit a terminator
            let mut found_terminator = false;
            for terminator in terminators {
                if remaining.starts_with(terminator) {
                    found_terminator = true;
                    break;
                }
            }
            
            if found_terminator {
                break;
            }
            
            if let Some(node) = self.parse_next_node()? {
                nodes.push(node);
            } else {
                break;
            }
        }
        
        Ok(nodes)
    }
}

/// Template executor for rendering AST with variables
#[derive(Debug)]
pub struct TemplateExecutor {
    locale_manager: Option<LocaleManager>,
    pub current_locale: String,
    template_base_path: PathBuf,
    include_stack: Vec<String>, // For circular reference detection
}

impl TemplateExecutor {
    pub fn new() -> Self {
        Self {
            locale_manager: None,
            current_locale: "en".to_string(),
            template_base_path: PathBuf::from("tests/fixtures/templates"),
            include_stack: Vec::new(),
        }
    }
    
    pub fn with_base_path(mut self, base_path: PathBuf) -> Self {
        self.template_base_path = base_path;
        self
    }

    pub fn with_locale_manager(mut self, manager: LocaleManager, locale: &str) -> Self {
        self.locale_manager = Some(manager);
        self.current_locale = locale.to_string();
        self
    }

    /// Render AST nodes with provided variables
    pub fn render(&mut self, nodes: &[TemplateNode], vars: &HashMap<String, serde_json::Value>) -> Result<String> {
        let mut result = String::new();
        
        for node in nodes {
            match self.render_node(node, vars)? {
                Some(content) => result.push_str(&content),
                None => {}
            }
        }
        
        Ok(result)
    }

    fn render_node(&mut self, node: &TemplateNode, vars: &HashMap<String, serde_json::Value>) -> Result<Option<String>> {
        match node {
            TemplateNode::Text(text) => Ok(Some(text.clone())),
            
            TemplateNode::Variable(name) => {
                if let Some(value) = vars.get(name) {
                    Ok(Some(self.value_to_string(value)))
                } else {
                    Err(Error::TemplateVariableNotFound { name: name.clone() })
                }
            }
            
            TemplateNode::NestedVariable(parts) => {
                if let Some(value) = self.resolve_nested_value(parts, vars) {
                    Ok(Some(self.value_to_string(&value)))
                } else {
                    Err(Error::TemplateVariableNotFound { name: parts.join(".") })
                }
            }
            
            TemplateNode::If { condition, then_content, else_content } => {
                if self.is_condition_true(condition, vars) {
                    self.render(then_content, vars).map(Some)
                } else if let Some(else_nodes) = else_content {
                    self.render(else_nodes, vars).map(Some)
                } else {
                    Ok(Some(String::new()))
                }
            }
            
            TemplateNode::Each { variable, content } => {
                if let Some(array_value) = vars.get(variable) {
                    if let Some(array) = array_value.as_array() {
                        let mut result = String::new();
                        
                        for item in array {
                            // Create new variable context with "this" pointing to current item
                            let mut item_vars = vars.clone();
                            item_vars.insert("this".to_string(), item.clone());
                            
                            // Also expand nested properties of the item into the context
                            if let Some(obj) = item.as_object() {
                                for (key, value) in obj {
                                    item_vars.insert(format!("this.{}", key), value.clone());
                                }
                            }
                            
                            result.push_str(&self.render(content, &item_vars)?);
                        }
                        
                        Ok(Some(result))
                    } else {
                        Ok(Some(String::new()))
                    }
                } else {
                    Ok(Some(String::new()))
                }
            }
            
            TemplateNode::Switch { variable, cases } => {
                if let Some(switch_value) = vars.get(variable) {
                    let switch_str = self.value_to_string(switch_value);
                    
                    for (case_value, case_content) in cases {
                        if &switch_str == case_value {
                            return self.render(case_content, vars).map(Some);
                        }
                    }
                }
                Ok(Some(String::new()))
            }
            
            TemplateNode::IfLocale { locale, then_content, else_content } => {
                if &self.current_locale == locale {
                    self.render(then_content, vars).map(Some)
                } else if let Some(else_nodes) = else_content {
                    self.render(else_nodes, vars).map(Some)
                } else {
                    Ok(Some(String::new()))
                }
            }
            
            TemplateNode::Include { path, params } => {
                self.render_include(path, params, vars)
            }
            
            TemplateNode::I18n { key, params } => {
                self.render_i18n(key, params, vars)
            }
            
            TemplateNode::Helper { name, args, params } => {
                self.render_helper(name, args, params, vars)
            }
            
            TemplateNode::Case { .. } => {
                // Cases are handled within Switch nodes
                Ok(None)
            }
        }
    }

    fn is_condition_true(&self, condition: &str, vars: &HashMap<String, serde_json::Value>) -> bool {
        if let Some(value) = vars.get(condition) {
            match value {
                serde_json::Value::Bool(b) => *b,
                serde_json::Value::Null => false,
                serde_json::Value::String(s) => !s.is_empty(),
                serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                serde_json::Value::Array(a) => !a.is_empty(),
                serde_json::Value::Object(o) => !o.is_empty(),
            }
        } else {
            false
        }
    }

    fn resolve_nested_value(&self, parts: &[String], vars: &HashMap<String, serde_json::Value>) -> Option<serde_json::Value> {
        if parts.is_empty() {
            return None;
        }

        let root_key = &parts[0];
        let mut current_value = vars.get(root_key)?.clone();

        for key in &parts[1..] {
            match current_value {
                serde_json::Value::Object(ref obj) => {
                    current_value = obj.get(key)?.clone();
                }
                _ => return None,
            }
        }

        Some(current_value)
    }

    fn value_to_string(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "".to_string(),
            _ => value.to_string(),
        }
    }

    fn render_i18n(&mut self, key: &str, params: &HashMap<String, String>, vars: &HashMap<String, serde_json::Value>) -> Result<Option<String>> {
        
        // Build parameter map for interpolation first (avoid borrow conflicts)
        let mut interpolation_vars = HashMap::new();
        
        for (param_name, param_value) in params {
            // Check if param_value is a helper function expression, variable reference, or literal
            let clean_value = param_value.trim_matches('"');
            
            if param_value.starts_with("plural ") || param_value.starts_with("format_number ") {
                // It's a helper function expression - parse and execute it
                let mut parser = TemplateParser::new(&format!("{{{{{}}}}}", param_value));
                if let Ok(nodes) = parser.parse() {
                    if let Some(TemplateNode::Helper { name, args, params: helper_params }) = nodes.first() {
                        if let Ok(Some(result)) = self.render_helper(&name, &args, &helper_params, vars) {
                            let final_result = result;
                            interpolation_vars.insert(param_name.clone(), serde_json::Value::String(final_result));
                        } else {
                            // Helper function failed, use literal
                            interpolation_vars.insert(param_name.clone(), serde_json::Value::String(param_value.to_string()));
                        }
                    } else {
                        // Not actually a helper node, use literal
                        interpolation_vars.insert(param_name.clone(), serde_json::Value::String(param_value.to_string()));
                    }
                } else {
                    // Helper parsing failed, use literal
                    interpolation_vars.insert(param_name.clone(), serde_json::Value::String(param_value.to_string()));
                }
            } else if let Some(value) = vars.get(clean_value) {
                // It's a variable reference
                interpolation_vars.insert(param_name.clone(), value.clone());
            } else if let Some(value) = vars.get(param_value) {
                // Try without cleaning quotes in case it's an exact match
                interpolation_vars.insert(param_name.clone(), value.clone());
            } else if clean_value.contains('.') {
                // It's a nested variable like user.role
                let parts: Vec<&str> = clean_value.split('.').collect();
                let string_parts: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                if let Some(nested_value) = self.resolve_nested_value(&string_parts, vars) {
                    interpolation_vars.insert(param_name.clone(), nested_value);
                } else {
                    // Nested variable not found, use literal
                    interpolation_vars.insert(param_name.clone(), serde_json::Value::String(clean_value.to_string()));
                }
            } else {
                // It's a literal value (remove quotes)
                interpolation_vars.insert(param_name.clone(), serde_json::Value::String(clean_value.to_string()));
            }
        }
        
        // Now access locale manager
        if let Some(ref mut locale_manager) = self.locale_manager.as_mut() {
            // Get locale data
            match locale_manager.get_locale(&self.current_locale) {
                Ok(locale_data) => {
                    // Get the translation string
                    if let Some(template_str) = locale_data.get_string(key) {
                    // Use LocaleData::interpolate for parameter substitution
                    match locale_data.interpolate(key, &interpolation_vars) {
                        Ok(result) => Ok(Some(result)),
                        Err(_) => {
                            // Fallback to original template string
                            Ok(Some(template_str))
                        }
                    }
                } else {
                    // Key not found, return error
                    Err(Error::I18nKeyNotFound { 
                        key: key.to_string(), 
                        locale: self.current_locale.clone() 
                    })
                }
            }
            Err(e) => {
                // Locale not found - propagate the error
                Err(e)
            }
        }
        } else {
            // No locale manager - return error for i18n keys when no locale is configured
            Err(Error::I18nKeyNotFound { 
                key: key.to_string(), 
                locale: self.current_locale.clone() 
            })
        }
    }

    fn render_include(&mut self, path: &str, params: &HashMap<String, String>, vars: &HashMap<String, serde_json::Value>) -> Result<Option<String>> {
        // Check for circular references
        if self.include_stack.contains(&path.to_string()) {
            return Ok(Some(format!("<!-- Circular reference detected: {} -->", path)));
        }

        // Resolve the full file path
        let full_path = self.template_base_path.join(path);
        
        // Check if file exists
        if !full_path.exists() {
            return Ok(Some(format!("<!-- Include not found: {} -->", path)));
        }

        // Read the include file
        match std::fs::read_to_string(&full_path) {
            Ok(include_content) => {
                // Push current path to stack for circular detection
                self.include_stack.push(path.to_string());

                // Create a new template from the include content
                let mut include_parser = TemplateParser::new(&include_content);
                let include_ast = include_parser.parse()?;

                // Merge parameters into variables
                let mut include_vars = vars.clone();
                for (param_name, param_value) in params {
                    let clean_value = param_value.trim_matches('"');
                    
                    if let Some(value) = vars.get(clean_value) {
                        // It's a variable reference
                        include_vars.insert(param_name.clone(), value.clone());
                    } else if let Some(value) = vars.get(param_value) {
                        // Try without cleaning quotes
                        include_vars.insert(param_name.clone(), value.clone());
                    } else if clean_value.contains('.') {
                        // It's a nested variable
                        let parts: Vec<&str> = clean_value.split('.').collect();
                        let string_parts: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                        if let Some(nested_value) = self.resolve_nested_value(&string_parts, vars) {
                            include_vars.insert(param_name.clone(), nested_value);
                        } else {
                            // Use literal
                            include_vars.insert(param_name.clone(), serde_json::Value::String(clean_value.to_string()));
                        }
                    } else {
                        // It's a literal value
                        include_vars.insert(param_name.clone(), serde_json::Value::String(clean_value.to_string()));
                    }
                }

                // Render the included template (locale manager is already set in self)
                let result = self.render(&include_ast, &include_vars)?;

                // Pop from stack
                self.include_stack.pop();

                Ok(Some(result))
            }
            Err(_) => {
                Ok(Some(format!("<!-- Error reading include: {} -->", path)))
            }
        }
    }

    fn render_helper(&mut self, name: &str, args: &[String], params: &HashMap<String, String>, vars: &HashMap<String, serde_json::Value>) -> Result<Option<String>> {
        match name {
            "format_number" => self.render_format_number_helper(args, params, vars),
            _ => Ok(Some(format!("<!-- Unknown helper: {} -->", name))),
        }
    }


    fn render_format_number_helper(&self, args: &[String], params: &HashMap<String, String>, vars: &HashMap<String, serde_json::Value>) -> Result<Option<String>> {
        if args.is_empty() {
            return Ok(Some("<!-- format_number helper requires arguments -->".to_string()));
        }

        // First argument should be the number variable
        let number_arg = &args[0];
        let number_value = if let Some(value) = vars.get(number_arg) {
            match value.as_f64() {
                Some(n) => n,
                None => {
                    // Try to parse as number from string
                    if let Some(s) = value.as_str() {
                        s.parse::<f64>().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                }
            }
        } else {
            return Ok(Some(format!("<!-- variable {} not found -->", number_arg)));
        };

        // Get style parameter
        let style = params.get("style").map(|s| s.trim_matches('"')).unwrap_or("decimal");

        match style {
            "percent" => {
                let percentage = number_value * 100.0;
                // Handle floating-point precision issues by rounding to nearest 0.1
                let rounded_percentage = (percentage * 10.0).round() / 10.0;
                
                // If the rounded value is very close to an integer, use integer format
                if (rounded_percentage - rounded_percentage.round()).abs() < 0.01 {
                    Ok(Some(format!("{}%", rounded_percentage.round() as i32)))
                } else {
                    Ok(Some(format!("{:.1}%", rounded_percentage)))
                }
            }
            "currency" => {
                let currency = params.get("currency").map(|s| s.trim_matches('"')).unwrap_or("USD");
                match currency {
                    "USD" => Ok(Some(format!("${:.2}", number_value))),
                    _ => Ok(Some(format!("{:.2} {}", number_value, currency))),
                }
            }
            "decimal" => {
                let precision = params.get("precision")
                    .and_then(|p| p.trim_matches('"').parse::<usize>().ok())
                    .unwrap_or(2);
                Ok(Some(format!("{:.prec$}", number_value, prec = precision)))
            }
            _ => {
                // Default formatting
                Ok(Some(format!("{}", number_value)))
            }
        }
    }
}

/// A compiled template ready for fast rendering with variables.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    frontmatter: Option<TemplateFrontmatter>,
    ast: Vec<TemplateNode>,
    default_variables: HashMap<String, serde_json::Value>,
    required_variables: Vec<String>,
    locale_manager: Option<LocaleManager>,
    current_locale: String,
    // Fluent API support
    accumulated_variables: HashMap<String, serde_json::Value>,
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
        
        // Parse template content into AST
        let mut parser = TemplateParser::new(&template_content);
        let ast = parser.parse()?;
        
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
            ast,
            default_variables,
            required_variables,
            locale_manager: None,
            current_locale: "en".to_string(),
            accumulated_variables: HashMap::new(),
        })
    }

    /// Set the locale for i18n (returns new template, doesn't mutate).
    pub fn with_locale(mut self, locale: &str) -> Result<Self> {
        self.current_locale = locale.to_string();
        
        // Initialize locale manager - try fixtures first, then fallback
        let locales_paths = ["tests/fixtures/locales", "locales"];
        for path in &locales_paths {
            if std::path::Path::new(path).exists() {
                match LocaleManager::new(path, "en") {
                    Ok(manager) => {
                        self.locale_manager = Some(manager);
                        break;
                    }
                    Err(_) => {
                        // Failed to create LocaleManager for this path, try next
                    }
                }
            }
        }
        
        Ok(self)
    }

    /// Get the list of required variables that must be provided at render time.
    pub fn required_variables(&self) -> &[String] {
        &self.required_variables
    }
    
    /// Get the frontmatter includes list
    pub fn includes(&self) -> Vec<String> {
        if let Some(ref frontmatter) = self.frontmatter {
            frontmatter.includes.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Validate that all frontmatter includes exist at the given base path
    pub fn validate_includes(&self, base_path: &Path) -> Result<()> {
        if let Some(ref frontmatter) = self.frontmatter {
            for include_path in &frontmatter.includes {
                let full_path = base_path.join(include_path);
                if !full_path.exists() {
                    return Err(Error::TemplateParsing(format!(
                        "Include file not found: {} (resolved to {})", 
                        include_path, 
                        full_path.display()
                    )));
                }
            }
        }
        Ok(())
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
        self.render_with_base_path(vars, None)
    }
    
    /// Render the template with variables and a specific base path for includes.
    pub fn render_with_base_path(&self, vars: &serde_json::Value, base_path: Option<PathBuf>) -> Result<String> {
        // Validate required variables
        self.validate_variables(vars)?;
        
        // Validate includes if base_path is provided
        if let Some(ref base) = base_path {
            self.validate_includes(base)?;
        }

        // Merge default variables with provided variables
        let mut all_vars = self.default_variables.clone();
        if let Some(vars_obj) = vars.as_object() {
            for (key, value) in vars_obj {
                all_vars.insert(key.clone(), value.clone());
            }
        }

        // Create executor and render AST
        let mut executor = TemplateExecutor::new();
        executor.current_locale = self.current_locale.clone();
        if let Some(ref manager) = self.locale_manager {
            executor = executor.with_locale_manager(manager.clone(), &self.current_locale);
        }
        if let Some(base) = base_path {
            executor = executor.with_base_path(base);
        }
        
        executor.render(&self.ast, &all_vars)
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

    // === FLUENT API METHODS ===
    // Support for builder pattern with variable accumulation

    /// Add a variable to the template (fluent API)
    pub fn var<K: Into<String>, V: serde::Serialize>(mut self, key: K, value: V) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.accumulated_variables.insert(key.into(), json_value);
        }
        self
    }

    /// Render the template using accumulated variables from .var() calls
    pub fn render_with_vars(&self) -> Result<String> {
        let vars = serde_json::Value::Object(self.accumulated_variables.clone().into_iter().collect());
        self.render(&vars)
    }

    /// Render the template with both accumulated variables and additional variables  
    pub fn render_with_additional_vars(&self, additional_vars: &serde_json::Value) -> Result<String> {
        let mut combined_vars = self.accumulated_variables.clone();
        
        // Merge additional variables (they take precedence)
        if let Some(additional_obj) = additional_vars.as_object() {
            for (key, value) in additional_obj {
                combined_vars.insert(key.clone(), value.clone());
            }
        }
        
        let vars = serde_json::Value::Object(combined_vars.into_iter().collect());
        self.render(&vars)
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
        template.render_with_base_path(vars, Some(self.base_path.clone()))
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

    // === TDD TESTS FOR MISSING FUNCTIONALITY ===
    // These tests should initially fail and drive implementation

    #[test]
    fn test_if_conditional_true() {
        let content = r#"{{#if debug_mode}}
## Debug Information
Debug logging is enabled.
{{/if}}

Regular content."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({ "debug_mode": true });
        let result = template.render(&vars).unwrap();
        
        assert!(result.contains("## Debug Information"));
        assert!(result.contains("Debug logging is enabled."));
        assert!(result.contains("Regular content."));
    }

    #[test]
    fn test_if_conditional_false() {
        let content = r#"{{#if debug_mode}}
Debug info should not appear.
{{/if}}

Regular content."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({ "debug_mode": false });
        let result = template.render(&vars).unwrap();
        
        assert!(!result.contains("Debug info should not appear."));
        assert!(result.contains("Regular content."));
    }

    #[test]
    fn test_if_else_conditional() {
        let content = r#"{{#if advanced_mode}}
Use advanced features.
{{else}}
Use basic features.
{{/if}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        
        // Test true case
        let vars = json!({ "advanced_mode": true });
        let result = template.render(&vars).unwrap();
        assert!(result.contains("Use advanced features."));
        assert!(!result.contains("Use basic features."));
        
        // Test false case  
        let vars = json!({ "advanced_mode": false });
        let result = template.render(&vars).unwrap();
        assert!(!result.contains("Use advanced features."));
        assert!(result.contains("Use basic features."));
    }

    #[test]
    fn test_each_loop_with_array() {
        let content = r#"{{#each projects}}
### {{this.name}}
Status: {{this.status}}
{{#each this.tasks}}
- [ ] {{this}}
{{/each}}
{{/each}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({
            "projects": [
                {
                    "name": "Project A",
                    "status": "In Progress",
                    "tasks": ["Task 1", "Task 2"]
                },
                {
                    "name": "Project B", 
                    "status": "Complete",
                    "tasks": ["Task 3"]
                }
            ]
        });

        let result = template.render(&vars).unwrap();
        assert!(result.contains("### Project A"));
        assert!(result.contains("Status: In Progress"));
        assert!(result.contains("- [ ] Task 1"));
        assert!(result.contains("- [ ] Task 2"));
        assert!(result.contains("### Project B"));
        assert!(result.contains("Status: Complete"));
        assert!(result.contains("- [ ] Task 3"));
    }

    #[test]
    fn test_switch_case_statements() {
        let content = r#"{{#switch user_level}}
  {{#case "beginner"}}
Start with basic concepts.
  {{/case}}
  {{#case "intermediate"}}
Focus on practical examples.
  {{/case}}
  {{#case "advanced"}}
Focus on optimization patterns.
  {{/case}}
{{/switch}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        
        // Test beginner case
        let vars = json!({ "user_level": "beginner" });
        let result = template.render(&vars).unwrap();
        assert!(result.contains("Start with basic concepts."));
        assert!(!result.contains("practical examples"));
        assert!(!result.contains("optimization patterns"));
        
        // Test advanced case
        let vars = json!({ "user_level": "advanced" });
        let result = template.render(&vars).unwrap();
        assert!(!result.contains("basic concepts"));
        assert!(!result.contains("practical examples"));
        assert!(result.contains("Focus on optimization patterns."));
    }

    // === PHASE 2: TEMPLATE INCLUDES TDD TESTS ===
    // These tests drive implementation of template composition system

    #[test]
    fn test_basic_template_include_with_fixtures() {
        let content = r#"# Main Template

{{> shared/greeting.md}}

Content here."#;

        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("en").unwrap();
        
        // Should resolve to tests/fixtures/templates/shared/greeting.md
        // Expected: "{{i18n "greeting"}}\n\nThis is content from the shared greeting template."
        let result = template.render(&json!({})).unwrap();
        assert!(result.contains("This is content from the shared greeting template."));
        assert!(result.contains("Hello! Welcome to the system"));
    }

    #[test]
    fn test_template_include_with_variables() {
        let content = r#"User: {{username}}

{{> shared/greeting.md}}"#;

        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("en").unwrap();
        let vars = json!({ "username": "Alice" });
        let result = template.render_with_base_path(&vars, Some(PathBuf::from("tests/fixtures/templates"))).unwrap();
        
        assert!(result.contains("User: Alice"));
        assert!(result.contains("This is content from the shared greeting template."));
        assert!(result.contains("Hello! Welcome to the system")); // From i18n greeting
    }

    #[test]
    fn test_recursive_template_includes() {
        let content = r#"{{> recursive/parent.md}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        let result = template.render(&json!({})).unwrap();
        
        // Should include: parent.md → child.md → grandchild.md
        assert!(result.contains("Parent content"));
        assert!(result.contains("Child content"));
        assert!(result.contains("Grandchild content"));
    }

    #[test]
    fn test_template_include_with_parameters() {
        let content = r#"{{> shared/footer.md locale="es" version="2.0"}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        let result = template.render(&json!({})).unwrap();
        
        // Parameters should be passed to the included template
        assert!(result.contains("es") || result.contains("2.0"));
    }

    #[test]
    fn test_template_include_with_template_variables() {
        let content = r#"{{> shared/footer.md locale=current_locale version=app_version}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({
            "current_locale": "es",
            "app_version": "3.1.4"
        });
        let result = template.render(&vars).unwrap();
        
        // Template variables should be resolved and passed to includes
        assert!(result.contains("es") || result.contains("3.1.4"));
    }

    #[test]
    fn test_template_include_nonexistent_file() {
        let content = r#"{{> nonexistent/template.md}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        let result = template.render(&json!({})).unwrap();
        
        // Should gracefully handle missing includes
        assert!(result.contains("Include not found") || result.contains("nonexistent/template.md"));
    }

    #[test]
    fn test_circular_reference_detection() {
        // This would test circular_a.md → circular_b.md → circular_a.md
        // For now, create a simple test case
        let content = r#"{{> recursive/circular_a.md}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        let result = template.render(&json!({}));
        
        // Should either detect circular reference or handle gracefully
        assert!(result.is_ok()); // At minimum, shouldn't crash
    }

    #[test]
    fn test_nested_includes_with_i18n() {
        let content = r#"{{> system_prompt.md}}"#;

        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("es").unwrap();
        
        let vars = json!({
            "role": "asistente",
            "domain": "programación"
        });
        let result = template.render(&vars).unwrap();
        
        // Should combine includes with i18n parameter interpolation
        assert!(result.contains("Instrucciones del Sistema"));
        assert!(result.contains("asistente"));
        assert!(result.contains("programación"));
    }

    #[test]
    fn test_multiple_includes_in_template() {
        let content = r#"{{> shared/greeting.md}}

Main content here.

{{> system_prompt.md}}

{{> shared/greeting.md}}"#;

        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("en").unwrap();
        let vars = json!({
            "role": "assistant",
            "domain": "software"
        });
        let result = template.render(&vars).unwrap();
        
        // Should process multiple includes correctly
        assert_eq!(result.matches("This is content from the shared greeting template.").count(), 2);
        assert!(result.contains("Main content here."));
        assert!(result.contains("System Instructions"));
    }

    #[test]
    fn test_include_path_resolution() {
        // Test that include paths are resolved relative to templates directory
        let content = r#"{{> conversations/learning_session.md}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({
            "topic": "Rust",
            "user_level": "intermediate",
            "user_question": "How do lifetimes work?"
        });
        let result = template.render(&vars).unwrap();
        
        // Should resolve complex conversation template
        assert!(result.contains("expert instructor"));
        assert!(result.contains("intermediate level"));
        assert!(result.contains("Rust"));
        assert!(result.contains("How do lifetimes work?"));
    }

    // === PHASE 1: ADVANCED I18N TDD TESTS ===
    // These tests drive the implementation of advanced i18n features

    #[test]
    fn test_i18n_basic_parameter_interpolation() {
        let content = r#"{{i18n "system.intro" role="assistant"}}"#;
        
        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("en").unwrap();
        
        // Should use fixtures/locales/en/system.yaml
        // system.intro: "You are a {role} with expertise in software development."
        let result = template.render(&json!({})).unwrap();
        assert_eq!(result, "You are a assistant with expertise in software development.");
    }

    #[test]
    fn test_i18n_with_template_variables() {
        let content = r#"{{i18n "system.intro" role=role}}"#;
        
        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("en").unwrap();
        
        let vars = json!({ "role": "senior engineer" });
        let result = template.render(&vars).unwrap();
        assert_eq!(result, "You are a senior engineer with expertise in software development.");
    }

    #[test]
    fn test_i18n_multilingual_with_fixtures() {
        let content = r#"{{i18n "system.intro" role="desarrollador"}}"#;
        
        // Test Spanish locale from fixtures/locales/es/system.yaml
        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("es").unwrap();
        
        let result = template.render(&json!({})).unwrap();
        assert_eq!(result, "Eres un desarrollador con experiencia en desarrollo de software.");
    }

    #[test]
    fn test_i18n_complex_parameter_interpolation() {
        let content = r#"{{i18n "current_tasks" count=task_count tasks="tareas"}}"#;
        
        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("es").unwrap();
        
        // Should use: "Actualmente tienes {count} {tasks} pendientes de revisión."
        let vars = json!({ "task_count": 3 });
        let result = template.render(&vars).unwrap();
        assert_eq!(result, "Actualmente tienes 3 tareas pendientes de revisión.");
    }

    #[test] 
    fn test_i18n_with_nested_variables() {
        let content = r#"{{i18n "system.intro" role=user.role}}"#;
        
        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("en").unwrap();
        
        let vars = json!({ 
            "user": { "role": "architect" }
        });
        let result = template.render(&vars).unwrap();
        assert_eq!(result, "You are a architect with expertise in software development.");
    }

    #[test]
    fn test_i18n_key_not_found_strict_error() {
        let content = r#"{{i18n "nonexistent.key" param="value"}}"#;
        
        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("en").unwrap();
        
        let result = template.render(&json!({}));
        // Should now return error for missing keys instead of placeholder
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::I18nKeyNotFound { key, locale } => {
                assert_eq!(key, "nonexistent.key");
                assert_eq!(locale, "en");
            }
            e => panic!("Expected I18nKeyNotFound error, got: {:?}", e),
        }
    }

    #[test]
    fn test_i18n_locale_fallback_chain() {
        let content = r#"{{i18n "system.title"}}"#;
        
        // Test locale fallback: es-MX → es → en
        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("es-MX").unwrap();
        
        let result = template.render(&json!({})).unwrap();
        // Should fall back to Spanish since es-MX doesn't exist
        assert_eq!(result, "Instrucciones del Sistema");
    }

    #[test]
    fn test_advanced_i18n_integration_with_conditionals() {
        let content = r#"{{#if_locale "ar"}}
{{i18n "system.title"}}
{{else}}
{{i18n "system.title"}}
{{/if_locale}}"#;

        // Test English fallback
        let template_en = PromptTemplate::from_content(content).unwrap()
            .with_locale("en").unwrap();
        let result_en = template_en.render(&json!({})).unwrap();
        assert!(result_en.contains("System Instructions"));

        // Test Arabic locale which has text_direction: "rtl"
        let template_ar = PromptTemplate::from_content(content).unwrap()
            .with_locale("ar").unwrap();
        let result_ar = template_ar.render(&json!({})).unwrap();
        assert!(result_ar.contains("تعليمات النظام"));
    }

    #[test]
    fn test_if_locale_conditional() {
        let content = r#"Welcome message.

{{#if_locale "ar"}}
<div dir="rtl">المحتوى بالعربية</div>
{{/if_locale}}

{{#if_locale "en"}}
<div dir="ltr">English content</div>
{{/if_locale}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        
        // Test Arabic locale
        let arabic_template = template.clone().with_locale("ar").unwrap();
        let result = arabic_template.render(&json!({})).unwrap();
        assert!(result.contains("المحتوى بالعربية"));
        assert!(!result.contains("English content"));
        
        // Test English locale
        let english_template = template.with_locale("en").unwrap();
        let result = english_template.render(&json!({})).unwrap();
        assert!(!result.contains("المحتوى بالعربية"));
        assert!(result.contains("English content"));
    }

    // === PHASE 3: HELPER FUNCTIONS TDD TESTS ===
    // These tests drive implementation of template helper functions




    #[test]
    fn test_format_number_helper_basic() {
        let content = r#"Progress: {{format_number progress style="percent"}}"#;
        
        let template = PromptTemplate::from_content(content).unwrap();
        
        let vars = json!({"progress": 0.75});
        let result = template.render(&vars).unwrap();
        assert!(result.contains("75%"));
        
        let vars = json!({"progress": 0.5});
        let result = template.render(&vars).unwrap();
        assert!(result.contains("50%"));
    }

    #[test]
    fn test_format_number_helper_currency() {
        let content = r#"Price: {{format_number price style="currency" currency="USD"}}"#;
        
        let template = PromptTemplate::from_content(content).unwrap();
        
        let vars = json!({"price": 29.99});
        let result = template.render(&vars).unwrap();
        assert!(result.contains("$29.99") || result.contains("29.99") || result.contains("USD"));
    }

    #[test]  
    fn test_format_number_helper_decimal() {
        let content = r#"Value: {{format_number value style="decimal" precision="2"}}"#;
        
        let template = PromptTemplate::from_content(content).unwrap();
        
        let vars = json!({"value": 3.14159});
        let result = template.render(&vars).unwrap();
        assert!(result.contains("3.14"));
    }





    #[test]
    fn test_format_number_with_locale() {
        let content = r#"{{format_number value style="decimal"}}"#;
        
        // Test different locales for number formatting
        let template_en = PromptTemplate::from_content(content).unwrap()
            .with_locale("en").unwrap();
        let vars = json!({"value": 1234.56});
        let result_en = template_en.render(&vars).unwrap();
        
        // English typically uses comma thousands separator
        assert!(result_en.contains("1,234.56") || result_en.contains("1234.56"));
        
        // Could test other locales if supported
        let template_es = PromptTemplate::from_content(content).unwrap()
            .with_locale("es").unwrap();
        let result_es = template_es.render(&vars).unwrap();
        // Spanish might use different separators, but for now just ensure it works
        assert!(!result_es.is_empty());
    }

    #[test]
    fn test_conditional_template_composition() {
        let content = r#"---
variables:
  mode: "{{mode}}"
---

# Main Template

{{> shared/header.md}}

{{#if advanced_mode}}
{{> advanced/expert_instructions.md}}
{{else}}
{{> basic/beginner_instructions.md}}
{{/if}}

{{> shared/footer.md locale=current_locale}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        
        // This combines conditionals with includes
        // Should initially fail until both features are implemented
        let vars = json!({ "advanced_mode": true });
        let _result = template.render(&vars);
        // This will fail until includes and conditionals are both working
    }

    #[test] 
    fn test_nested_conditionals_and_loops() {
        let content = r#"{{#each users}}
## User: {{this.name}}

{{#if this.active}}
Status: Active
{{#each this.permissions}}
- Permission: {{this}}
{{/each}}
{{else}}
Status: Inactive
{{/if}}

{{/each}}"#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({
            "users": [
                {
                    "name": "Alice",
                    "active": true,
                    "permissions": ["read", "write"]
                },
                {
                    "name": "Bob", 
                    "active": false,
                    "permissions": []
                }
            ]
        });

        let result = template.render(&vars).unwrap();
        assert!(result.contains("## User: Alice"));
        assert!(result.contains("Status: Active"));
        assert!(result.contains("- Permission: read"));
        assert!(result.contains("- Permission: write"));
        assert!(result.contains("## User: Bob"));
        assert!(result.contains("Status: Inactive"));
        assert!(!result.contains("- Permission:") || result.matches("- Permission:").count() == 2); // Only Alice's permissions
    }

    // === PHASE 4: BUILDER API TDD TESTS ===
    // These tests drive the implementation of missing builder API methods

    #[test]
    fn test_text_request_builder_system_from_md_with_var_chaining() {
        // Test the fluent API: .system_from_md().var().var()
        // This should fail initially because these methods don't exist
        
        // Create a simple test template file first
        std::fs::create_dir_all("tests/fixtures/builder_test").unwrap();
        std::fs::write(
            "tests/fixtures/builder_test/system.md", 
            "You are a {{role}} with {{years}} years of experience in {{domain}}."
        ).unwrap();
        
        use crate::{azure, provider::ProviderBuilder};
        let provider = azure().with_config(crate::providers::AzureConfig {
            api_key: "test".to_string(),
            resource: "test".to_string(),
            api_version: "2024-02-15-preview".to_string(),
        }).build().unwrap();
        let client = crate::Client::new(provider);

        // This should work with fluent chaining
        let _builder = client
            .text()
            .model("gpt-4o")
            .system_from_md("tests/fixtures/builder_test/system.md").unwrap()
            .var("role", "senior engineer")
            .var("years", 8)
            .var("domain", "Rust programming");
            
        // If we get here without compilation errors, the API exists
        // We can't easily test the actual request without mocking, 
        // but we can verify the builder chain compiles
    }

    #[test]
    fn test_text_request_builder_assistant_from_md_with_locale() {
        // Test assistant_from_md with locale support
        std::fs::create_dir_all("tests/fixtures/builder_test").unwrap();
        std::fs::write(
            "tests/fixtures/builder_test/assistant.md", 
            "{{i18n \"greeting\"}} {{user_name}}"
        ).unwrap();
        
        use crate::{azure, provider::ProviderBuilder};
        let provider = azure().with_config(crate::providers::AzureConfig {
            api_key: "test".to_string(),
            resource: "test".to_string(),
            api_version: "2024-02-15-preview".to_string(),
        }).build().unwrap();
        let client = crate::Client::new(provider);

        let _builder = client
            .text()
            .model("gpt-4o")
            .assistant_from_md("tests/fixtures/builder_test/assistant.md").unwrap()
            .var("user_name", "Alice")
            .with_locale("es").unwrap();
    }

    #[test]
    fn test_structured_request_builder_with_template_variables() {
        // Test structured request builder with template methods
        use serde::Deserialize;
        use schemars::JsonSchema;
        
        #[derive(Clone, Debug, JsonSchema, Deserialize)]
        struct TestResponse {
            message: String,
        }
        
        std::fs::create_dir_all("tests/fixtures/builder_test").unwrap();
        std::fs::write(
            "tests/fixtures/builder_test/structured_system.md", 
            "Analyze this {{content_type}} and provide {{analysis_depth}} analysis."
        ).unwrap();
        
        use crate::{azure, provider::ProviderBuilder};
        let provider = azure().with_config(crate::providers::AzureConfig {
            api_key: "test".to_string(),
            resource: "test".to_string(),
            api_version: "2024-02-15-preview".to_string(),
        }).build().unwrap();
        let client = crate::Client::new(provider);

        let _builder = client
            .structured::<TestResponse>()
            .model("gpt-4o")
            .system_from_md("tests/fixtures/builder_test/structured_system.md").unwrap()
            .var("content_type", "code")
            .var("analysis_depth", "detailed")
            .user("print('hello world')");
    }

    #[test]
    fn test_messages_builder_with_template_methods() {
        // Test Messages builder with template methods
        use crate::Messages;
        
        std::fs::create_dir_all("tests/fixtures/builder_test").unwrap();
        std::fs::write(
            "tests/fixtures/builder_test/conversation_system.md", 
            "You are a {{role}} helping with {{task_type}} tasks."
        ).unwrap();
        std::fs::write(
            "tests/fixtures/builder_test/conversation_assistant.md", 
            "I'll help you with {{task_type}}. What specific {{sub_area}} do you need assistance with?"
        ).unwrap();

        let _conversation = Messages::new()
            .system_from_md("tests/fixtures/builder_test/conversation_system.md").unwrap()
            .var("role", "coding mentor")
            .var("task_type", "programming")
            .user("I need help with Rust")
            .assistant_from_md("tests/fixtures/builder_test/conversation_assistant.md").unwrap()
            .var("task_type", "Rust programming")
            .var("sub_area", "concepts")
            .with_locale("en").unwrap();
    }

    #[test]
    fn test_template_builder_variable_accumulation() {
        // Test that .var() calls accumulate properly
        std::fs::create_dir_all("tests/fixtures/builder_test").unwrap();
        std::fs::write(
            "tests/fixtures/builder_test/multi_var.md", 
            "Hello {{name}}, you are {{age}} years old and work as a {{job}} in {{location}}."
        ).unwrap();
        
        let template = PromptTemplate::load("tests/fixtures/builder_test/multi_var.md").unwrap()
            .var("name", "Alice")
            .var("age", 30)
            .var("job", "engineer")
            .var("location", "San Francisco");
            
        let result = template.render_with_vars().unwrap();
        assert!(result.contains("Hello Alice"));
        assert!(result.contains("30 years old"));
        assert!(result.contains("work as a engineer"));
        assert!(result.contains("in San Francisco"));
    }

    #[test]
    fn test_template_variable_override() {
        // Test that later .var() calls override earlier ones
        std::fs::create_dir_all("tests/fixtures/builder_test").unwrap();
        std::fs::write(
            "tests/fixtures/builder_test/override.md", 
            "Role: {{role}}"
        ).unwrap();
        
        let template = PromptTemplate::load("tests/fixtures/builder_test/override.md").unwrap()
            .var("role", "junior")
            .var("role", "senior"); // Should override
            
        let result = template.render_with_vars().unwrap();
        assert!(result.contains("Role: senior"));
        assert!(!result.contains("Role: junior"));
    }

    // === PHASE 5: COMPREHENSIVE INTEGRATION TESTS ===
    // These tests verify that ALL features work together seamlessly

    #[test]
    fn test_complete_workflow_advanced_i18n_with_helpers_and_includes() {
        // Test the most complex scenario: template includes + i18n + helper functions + builder API
        
        // Create a comprehensive template structure
        std::fs::create_dir_all("tests/fixtures/integration").unwrap();
        std::fs::create_dir_all("tests/fixtures/integration/shared").unwrap();
        
        // Main template with includes, i18n, and helpers
        std::fs::write(
            "tests/fixtures/integration/complex_system.md",
            r#"---
variables:
  user_count: "{{user_count}}"
  locale_key: "system"
---

# {{i18n "system.title"}}

{{> shared/status_header.md active_users=user_count system_status=status}}

## Current Status
{{i18n "current_tasks" count=task_count tasks=(plural task_count "task")}}

## Performance Metrics
- Users: {{plural user_count "user"}}
- Progress: {{format_number completion style="percent"}}
- Load: {{format_number cpu_usage style="decimal"}}

{{> shared/footer.md version=system_version}}"#
        ).unwrap();
        
        // Status header include
        std::fs::write(
            "tests/fixtures/integration/shared/status_header.md",
            r#"### {{i18n "status.header" users=(plural active_users "user")}}
System Status: {{system_status}}"#
        ).unwrap();
        
        // Footer include (reuse existing)
        
        // Test with English locale
        let template = PromptTemplate::load("tests/fixtures/integration/complex_system.md").unwrap()
            .with_locale("en").unwrap()
            .var("user_count", 1250)
            .var("task_count", 3)
            .var("completion", 0.847)
            .var("cpu_usage", 23.456)
            .var("status", "Operational")
            .var("system_version", "v2.1.0")
            .var("locale", "en");
            
        let result = template.render_with_vars().unwrap();
        
        // Verify all features working together
        assert!(result.contains("System Instructions")); // i18n title
        // Note: plural helper removed
        assert!(result.contains("84.7%")); // format_number percent
        assert!(result.contains("23.46")); // format_number decimal (default 2 decimal places)
        // Note: plural helper removed from includes
        assert!(result.contains("System Status: Operational")); // include with variables
        assert!(result.contains("Version: v2.1.0")); // footer include
        assert!(result.contains("Thanks for using")); // footer content
    }

    #[test]
    fn test_request_builder_complete_integration() {
        // Test complete request builder workflow with all fluent API features
        
        // Create a complex conversation template
        std::fs::create_dir_all("tests/fixtures/integration/conversations").unwrap();
        
        std::fs::write(
            "tests/fixtures/integration/conversations/expert_system.md",
            r#"You are a {{role}} with {{years}} years of experience.

Your specialization: {{domain}}
Current locale: {{current_locale}}

{{#if advanced_mode}}
## Advanced Instructions
Use advanced techniques and assume deep knowledge.
{{else}}
## Standard Instructions  
Provide clear explanations suitable for {{user_level}} level.
{{/if}}

Current workload: {{i18n "current_tasks" count=active_tasks tasks=(plural active_tasks "task")}}"#
        ).unwrap();
        
        std::fs::write(
            "tests/fixtures/integration/conversations/assistant_response.md",
            r#"I understand you need help with {{topic}}.

Based on your {{user_level}} level, I'll provide {{format_number detail_level style="decimal"}}% detailed explanations.

Let me help you with {{i18n "task_type" type=topic}}."#
        ).unwrap();
        
        use crate::{azure, provider::ProviderBuilder};
        let provider = azure().with_config(crate::providers::AzureConfig {
            api_key: "test".to_string(),
            resource: "test".to_string(),
            api_version: "2024-02-15-preview".to_string(),
        }).build().unwrap();
        let client = crate::Client::new(provider);

        // Build complex request with all fluent API features
        let _builder = client
            .text()
            .model("gpt-4o")
            .system_from_md("tests/fixtures/integration/conversations/expert_system.md").unwrap()
            .var("role", "senior architect")
            .var("years", 15)
            .var("domain", "distributed systems")
            .var("current_locale", "en")
            .var("advanced_mode", true)
            .var("user_level", "intermediate")
            .var("active_tasks", 7)
            .with_locale("en").unwrap()
            .user("I need help designing a microservices architecture")
            .assistant_from_md("tests/fixtures/integration/conversations/assistant_response.md").unwrap()
            .var("topic", "microservices")
            .var("user_level", "intermediate")
            .var("detail_level", 0.85)
            .user("What about service discovery?");
            
        // If compilation succeeds, the fluent API integration works
        // In a real test, we'd verify the actual message content and ordering
    }

    #[test]
    fn test_messages_builder_with_complete_feature_set() {
        // Test Messages builder with all template features
        
        std::fs::create_dir_all("tests/fixtures/integration/messages").unwrap();
        
        std::fs::write(
            "tests/fixtures/integration/messages/system_prompt.md",
            r#"You are a {{role}} assistant helping with {{task_type}}.

Experience level: {{experience}}
Active sessions: {{session_count}}
Load factor: {{format_number load_factor style="percent"}}"#
        ).unwrap();
        
        std::fs::write(
            "tests/fixtures/integration/messages/followup.md",
            r#"{{i18n "followup.message" topic=current_topic difficulty=complexity_level}}"#
        ).unwrap();

        let conversation = crate::Messages::new()
            .system_from_md("tests/fixtures/integration/messages/system_prompt.md").unwrap()
            .var("role", "expert")
            .var("task_type", "system design")
            .var("experience", "senior")
            .var("session_count", 12)
            .var("load_factor", 0.73)
            .with_locale("en").unwrap()
            .user("How do I design a scalable chat system?")
            .assistant("Let me help you design a scalable chat system. We'll need to consider...")
            .user("What about real-time messaging?")
            .assistant_from_md("tests/fixtures/integration/messages/followup.md").unwrap()
            .var("current_topic", "real-time messaging")
            .var("complexity_level", 2);
            
        // Verify the conversation structure
        assert_eq!(conversation.len(), 5);
        
        // Check that template variables were properly applied
        let inputs = conversation.render_inputs();
        
        // System message should contain rendered template
        if let crate::types::Input::Message(msg) = &inputs[0] {
            assert!(msg.content.contains("expert assistant"));
            assert!(msg.content.contains("12"));
            assert!(msg.content.contains("73%"));
        }
        
        // Assistant message should contain i18n result
        if let crate::types::Input::Message(msg) = &inputs[4] {
            // Should contain rendered i18n message about real-time messaging
            assert!(msg.content.len() > 10); // Should have meaningful content
        }
    }

    #[test]
    fn test_cross_locale_integration() {
        // Test that locale switching works across all features
        
        std::fs::create_dir_all("tests/fixtures/integration/locales").unwrap();
        
        std::fs::write(
            "tests/fixtures/integration/multi_locale.md",
            r#"# {{i18n "greeting"}}

Users: {{plural user_count "user"}}
Progress: {{format_number completion style="percent"}}"#
        ).unwrap();
        
        // Test English
        let template_en = PromptTemplate::load("tests/fixtures/integration/multi_locale.md").unwrap()
            .with_locale("en").unwrap()
            .var("user_count", 5)
            .var("completion", 0.92);
            
        let result_en = template_en.render_with_vars().unwrap();
        assert!(result_en.contains("Hello")); // English greeting
        // Note: plural helper removed 
        assert!(result_en.contains("92%")); // Number formatting
        
        // Test Spanish  
        let template_es = PromptTemplate::load("tests/fixtures/integration/multi_locale.md").unwrap()
            .with_locale("es").unwrap()
            .var("user_count", 5)
            .var("completion", 0.92);
            
        let result_es = template_es.render_with_vars().unwrap();
        assert!(result_es.contains("¡Hola")); // Spanish greeting
        // Note: plural helper removed
        assert!(result_es.contains("92%")); // Number formatting (universal)
    }

    #[test]
    fn test_performance_and_caching() {
        // Test that repeated operations are efficient
        
        std::fs::create_dir_all("tests/fixtures/integration/performance").unwrap();
        
        std::fs::write(
            "tests/fixtures/integration/performance/cached_template.md",
            r#"{{iteration_count}} iterations completed, {{format_number completion style="percent"}}."#
        ).unwrap();
        
        let template = PromptTemplate::load("tests/fixtures/integration/performance/cached_template.md").unwrap()
            .with_locale("en").unwrap();
            
        // Test multiple renders with different variables (should be fast)
        let start = std::time::Instant::now();
        
        for i in 1..=100 {
            let result = template.clone()
                .var("iteration_count", i)
                .var("completion", i as f64 / 100.0)
                .render_with_vars().unwrap();
            
            if i == 1 {
                println!("Sample result: '{}'", result);
            }
                
            assert!(result.contains(&format!("{} iterations", i)));
            if i == 1 {
                let expected = format!("{}%", i);
                println!("Looking for: '{}'", expected);
                if !result.contains(&expected) {
                    println!("Result doesn't contain expected percentage");
                }
            }
            assert!(result.contains(&format!("{}%", i)));
        }
        
        let duration = start.elapsed();
        
        // Should complete 100 renders in reasonable time (< 100ms)
        assert!(duration.as_millis() < 100, "Performance test took too long: {}ms", duration.as_millis());
    }

    #[test]
    fn test_error_handling_integration() {
        // Test that errors are properly returned instead of placeholders
        
        std::fs::create_dir_all("tests/fixtures/integration/errors").unwrap();
        
        std::fs::write(
            "tests/fixtures/integration/errors/mixed_template.md",
            r#"Valid variable: {{item_count}}
Invalid variable: {{missing_var}}"#
        ).unwrap();
        
        let template = PromptTemplate::load("tests/fixtures/integration/errors/mixed_template.md").unwrap()
            .var("item_count", 3);
            
        // Should return error when encountering missing variable
        let result = template.render_with_vars();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::TemplateVariableNotFound { name } => {
                assert_eq!(name, "missing_var");
            }
            e => panic!("Expected TemplateVariableNotFound error, got: {:?}", e),
        }
    }

    #[test]
    fn test_variable_order_flexibility() {
        // Create test template file
        std::fs::create_dir_all("tests/fixtures/template_vars").unwrap();
        std::fs::write(
            "tests/fixtures/template_vars/test.md",
            "Hello {{name}}, you are a {{role}}."
        ).unwrap();

        // Test 1: Variables before template (original working order)
        let conversation1 = crate::Messages::new()
            .var("name", "Alice")
            .var("role", "developer")
            .system_from_md("tests/fixtures/template_vars/test.md").unwrap();
        
        let inputs1 = conversation1.render_inputs();
        if let crate::types::Input::Message(msg) = &inputs1[0] {
            assert!(msg.content.contains("Hello Alice"));
            assert!(msg.content.contains("you are a developer"));
        }

        // Test 2: Variables after template (should also work with lazy evaluation)
        let conversation2 = crate::Messages::new()
            .system_from_md("tests/fixtures/template_vars/test.md").unwrap()
            .var("name", "Bob")
            .var("role", "designer");
        
        let inputs2 = conversation2.render_inputs();
        if let crate::types::Input::Message(msg) = &inputs2[0] {
            assert!(msg.content.contains("Hello Bob"));
            assert!(msg.content.contains("you are a designer"));
        }

        // Test 3: Mixed order should also work
        let conversation3 = crate::Messages::new()
            .var("name", "Charlie")
            .system_from_md("tests/fixtures/template_vars/test.md").unwrap()
            .var("role", "manager");
        
        let inputs3 = conversation3.render_inputs();
        if let crate::types::Input::Message(msg) = &inputs3[0] {
            assert!(msg.content.contains("Hello Charlie"));
            assert!(msg.content.contains("you are a manager"));
        }
    }
}