#[cfg(test)]
mod tests {
    use responses::prompt::PromptTemplate;
    use responses::Error;
    use serde_json::json;

    #[test]
    fn test_error_on_missing_variable_not_in_required() {
        // Template has a variable that's not in required_variables list
        let content = r#"---
variables:
  role: "assistant"
---
Hello {{name}}, I am your {{role}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({});
        
        let result = template.render(&vars);
        assert!(result.is_err(), "Should error when variable is missing");
        
        match result.unwrap_err() {
            Error::TemplateVariableNotFound { name } => {
                assert_eq!(name, "name");
            }
            e => panic!("Expected TemplateVariableNotFound error, got: {:?}", e),
        }
    }

    #[test]
    fn test_error_on_missing_nested_variable() {
        // Template has nested variable that doesn't exist
        let content = r#"---
variables:
  role: "assistant"
---
Hello {{user.name}}, I am your {{role}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({});
        
        let result = template.render(&vars);
        assert!(result.is_err(), "Should error when nested variable is missing");
        
        match result.unwrap_err() {
            Error::TemplateVariableNotFound { name } => {
                assert_eq!(name, "user.name");
            }
            e => panic!("Expected TemplateVariableNotFound error, got: {:?}", e),
        }
    }

    #[test]
    fn test_error_on_partially_missing_nested_variable() {
        // Template has nested variable where parent exists but child doesn't
        let content = r#"---
variables:
  role: "assistant"
---
Hello {{user.name}}, I am your {{role}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({
            "user": {
                "id": 123
                // missing "name" field
            }
        });
        
        let result = template.render(&vars);
        assert!(result.is_err(), "Should error when nested variable field is missing");
        
        match result.unwrap_err() {
            Error::TemplateVariableNotFound { name } => {
                assert_eq!(name, "user.name");
            }
            e => panic!("Expected TemplateVariableNotFound error, got: {:?}", e),
        }
    }

    #[test]
    fn test_error_on_missing_i18n_key() {
        // Template with i18n key that doesn't exist
        let content = r#"---
variables:
  role: "assistant"
---
{{i18n "nonexistent.key"}} {{role}}!"#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({});
        
        let result = template.render(&vars);
        assert!(result.is_err(), "Should error when i18n key is missing");
        
        match result.unwrap_err() {
            Error::I18nKeyNotFound { key, locale } => {
                assert_eq!(key, "nonexistent.key");
                assert_eq!(locale, "en"); // default locale
            }
            e => panic!("Expected I18nKeyNotFound error, got: {:?}", e),
        }
    }

    #[test]
    fn test_error_on_missing_i18n_key_with_locale() {
        // Template with i18n key that doesn't exist in specific locale
        let content = r#"---
variables:
  role: "assistant"
i18n_key: "system"
---
{{i18n "missing.key"}} {{role}}!"#;

        let template = PromptTemplate::from_content(content).unwrap()
            .with_locale("es").unwrap();
        let vars = json!({});
        
        let result = template.render(&vars);
        assert!(result.is_err(), "Should error when i18n key is missing in locale");
        
        match result.unwrap_err() {
            Error::I18nKeyNotFound { key, locale } => {
                assert_eq!(key, "missing.key");
                assert_eq!(locale, "es");
            }
            e => panic!("Expected I18nKeyNotFound error, got: {:?}", e),
        }
    }

    #[test]
    fn test_successful_render_with_all_variables() {
        // Ensure normal operation still works
        let content = r#"---
variables:
  role: "assistant"
---
Hello {{name}}, I am your {{role}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({
            "name": "Alice"
        });
        
        let result = template.render(&vars);
        assert!(result.is_ok(), "Should succeed when all variables are provided");
        
        let rendered = result.unwrap();
        assert_eq!(rendered.trim(), "Hello Alice, I am your assistant.");
    }

    #[test]
    fn test_successful_render_with_nested_variables() {
        // Ensure nested variables work when properly provided
        let content = r#"---
variables:
  role: "assistant"
---
Hello {{user.name}}, I am your {{role}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({
            "user": {
                "name": "Bob"
            }
        });
        
        let result = template.render(&vars);
        assert!(result.is_ok(), "Should succeed when nested variables are provided");
        
        let rendered = result.unwrap();
        assert_eq!(rendered.trim(), "Hello Bob, I am your assistant.");
    }
}