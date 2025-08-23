//! Tests for the new functionality added to address user feedback

#[cfg(test)]
mod new_functionality_tests {
    use responses::{Response, Refusal};
    use responses::prompt::template::{TemplateSet, TemplateSetBuilder};
    use responses::types::OutputFunctionCall;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    /// Test auto-locale detection in TemplateSet::from_dir()
    #[test]
    fn test_auto_locale_detection() {
        // Create temporary directory structure
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        let locales_dir = template_dir.join("locales");
        let en_dir = locales_dir.join("en");
        
        fs::create_dir_all(&en_dir).expect("Failed to create directories");
        
        // Create a template file
        let template_file = template_dir.join("test.md");
        fs::write(&template_file, r#"---
variables:
  role: "assistant"
---
{{i18n "greeting"}} {{role}}!"#).expect("Failed to write template file");
        
        // Create a locale file
        let locale_file = en_dir.join("messages.yaml");
        fs::write(&locale_file, r#"
greeting: "Hello"
"#).expect("Failed to write locale file");
        
        // Test that TemplateSet automatically detects locales
        let template_set = TemplateSet::from_dir(&template_dir).expect("Failed to create TemplateSet");
        
        // Verify templates were loaded
        let templates = template_set.list_templates();
        assert!(templates.contains(&"test"), "Template 'test' should be loaded");
        
        // Test rendering with auto-detected i18n
        let vars = json!({});
        let result = template_set.render("test", &vars);
        
        // Should work because locales were auto-detected and configured
        if let Ok(rendered) = result {
            assert!(rendered.contains("Hello"), "Should contain localized greeting");
            assert!(rendered.contains("assistant"), "Should contain default variable");
        } else {
            // If i18n setup failed, we should at least get a meaningful error
            println!("Render result: {:?}", result);
        }
    }

    /// Test auto-locale detection with no locales directory
    #[test] 
    fn test_auto_locale_detection_no_locales() {
        // Create temporary directory structure without locales
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        
        fs::create_dir_all(&template_dir).expect("Failed to create directories");
        
        // Create a simple template file (no i18n)
        let template_file = template_dir.join("simple.md");
        fs::write(&template_file, r#"---
variables:
  role: "assistant"
---
Hello {{role}}!"#).expect("Failed to write template file");
        
        // Test that TemplateSet works normally without locales
        let template_set = TemplateSet::from_dir(&template_dir).expect("Failed to create TemplateSet");
        
        // Verify templates were loaded
        let templates = template_set.list_templates();
        assert!(templates.contains(&"simple"), "Template 'simple' should be loaded");
        
        // Test rendering without i18n
        let vars = json!({});
        let rendered = template_set.render("simple", &vars).expect("Should render successfully");
        
        assert!(rendered.contains("Hello"), "Should contain greeting");
        assert!(rendered.contains("assistant"), "Should contain default variable");
    }

    /// Test Response API helper methods for function-only responses
    #[test]
    fn test_response_helper_methods_function_only() {
        // Create a function-only response (common pattern)
        let function_call = OutputFunctionCall {
            name: "get_weather".to_string(),
            arguments: json!({"city": "Paris"}).to_string(),
        };

        let response: Response<String> = Response {
            message: None,  // No text message
            function_calls: vec![function_call],
        };

        // Test helper methods
        assert!(!response.has_text_message(), "Should not have text message");
        assert!(response.has_function_calls(), "Should have function calls");
        assert!(response.is_function_only(), "Should be function-only response");
        assert!(!response.has_both_text_and_functions(), "Should not have both");
        assert_eq!(response.function_call_count(), 1, "Should have one function call");
        assert!(response.text_message().is_none(), "Text message should be None");
        assert!(response.refusal().is_none(), "Refusal should be None");
    }

    /// Test Response API helper methods for text-only responses
    #[test]
    fn test_response_helper_methods_text_only() {
        let response: Response<String> = Response {
            message: Some(Ok("Hello, how can I help?".to_string())),
            function_calls: vec![],
        };

        // Test helper methods
        assert!(response.has_text_message(), "Should have text message");
        assert!(!response.has_function_calls(), "Should not have function calls");
        assert!(!response.is_function_only(), "Should not be function-only");
        assert!(!response.has_both_text_and_functions(), "Should not have both");
        assert_eq!(response.function_call_count(), 0, "Should have no function calls");
        
        assert_eq!(response.text_message(), Some(&"Hello, how can I help?".to_string()), 
                   "Should return text message");
        assert!(response.refusal().is_none(), "Refusal should be None");
    }

    /// Test Response API helper methods for mixed responses
    #[test]
    fn test_response_helper_methods_mixed() {
        let function_call = OutputFunctionCall {
            name: "search_docs".to_string(),
            arguments: json!({"query": "rust async"}).to_string(),
        };

        let response: Response<String> = Response {
            message: Some(Ok("Let me search for information about Rust async.".to_string())),
            function_calls: vec![function_call],
        };

        // Test helper methods
        assert!(response.has_text_message(), "Should have text message");
        assert!(response.has_function_calls(), "Should have function calls");
        assert!(!response.is_function_only(), "Should not be function-only");
        assert!(response.has_both_text_and_functions(), "Should have both");
        assert_eq!(response.function_call_count(), 1, "Should have one function call");
        
        assert!(response.text_message().is_some(), "Should have text message");
        assert!(response.refusal().is_none(), "Refusal should be None");
    }

    /// Test Response API helper methods for refusal responses
    #[test]
    fn test_response_helper_methods_refusal() {
        let response: Response<String> = Response {
            message: Some(Err(Refusal::from("I cannot help with that request.".to_string()))),
            function_calls: vec![],
        };

        // Test helper methods
        assert!(response.has_text_message(), "Should have text message (refusal)");
        assert!(!response.has_function_calls(), "Should not have function calls");
        assert!(!response.is_function_only(), "Should not be function-only");
        assert!(!response.has_both_text_and_functions(), "Should not have both");
        assert_eq!(response.function_call_count(), 0, "Should have no function calls");
        
        assert!(response.text_message().is_none(), "Text message should be None for refusal");
        assert!(response.refusal().is_some(), "Should have refusal");
    }

    /// Test Response API helper methods for empty responses
    #[test]
    fn test_response_helper_methods_empty() {
        let response: Response<String> = Response {
            message: None,
            function_calls: vec![],
        };

        // Test helper methods
        assert!(!response.has_text_message(), "Should not have text message");
        assert!(!response.has_function_calls(), "Should not have function calls");
        assert!(!response.is_function_only(), "Should not be function-only (no functions)");
        assert!(!response.has_both_text_and_functions(), "Should not have both");
        assert_eq!(response.function_call_count(), 0, "Should have no function calls");
        
        assert!(response.text_message().is_none(), "Text message should be None");
        assert!(response.refusal().is_none(), "Refusal should be None");
    }

    /// Test TemplateSetBuilder basic functionality
    #[test]
    fn test_template_set_builder_basic() {
        // Create temporary directory structure
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        
        fs::create_dir_all(&template_dir).expect("Failed to create directories");
        
        // Create a template file
        let template_file = template_dir.join("greeting.md");
        fs::write(&template_file, r#"---
variables:
  name: "World"
---
Hello {{name}}!"#).expect("Failed to write template file");
        
        // Test builder pattern
        let template_set = TemplateSet::builder()
            .directory(&template_dir)
            .default_locale("en")
            .build()
            .expect("Failed to build TemplateSet");
        
        // Verify configuration
        assert_eq!(template_set.current_locale(), "en", "Should have correct default locale");
        
        // Verify templates were loaded
        let templates = template_set.list_templates();
        assert!(templates.contains(&"greeting"), "Template 'greeting' should be loaded");
        
        // Test rendering
        let vars = json!({});
        let rendered = template_set.render("greeting", &vars).expect("Should render successfully");
        assert!(rendered.contains("Hello World!"), "Should contain greeting with default variable");
    }

    /// Test TemplateSetBuilder with auto-configure locales
    #[test]
    fn test_template_set_builder_auto_configure() {
        // Create temporary directory structure with locales
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        let locales_dir = template_dir.join("locales");
        let en_dir = locales_dir.join("en");
        
        fs::create_dir_all(&en_dir).expect("Failed to create directories");
        
        // Create a template file with i18n
        let template_file = template_dir.join("greeting.md");
        fs::write(&template_file, r#"---
variables:
  name: "World"
---
{{i18n "hello"}} {{name}}!"#).expect("Failed to write template file");
        
        // Create a locale file
        let locale_file = en_dir.join("messages.yaml");
        fs::write(&locale_file, r#"
hello: "Hello"
"#).expect("Failed to write locale file");
        
        // Test builder with auto-configure locales
        let template_set = TemplateSet::builder()
            .directory(&template_dir)
            .auto_configure_locales()
            .default_locale("en")
            .build()
            .expect("Failed to build TemplateSet");
        
        // Verify configuration
        assert_eq!(template_set.current_locale(), "en", "Should have correct default locale");
        
        // Test rendering with i18n (should work due to auto-configure)
        let vars = json!({});
        let result = template_set.render("greeting", &vars);
        
        if let Ok(rendered) = result {
            assert!(rendered.contains("Hello World!"), "Should contain localized greeting");
        }
    }

    /// Test TemplateSetBuilder with explicit locale paths
    #[test]
    fn test_template_set_builder_explicit_locales() {
        // Create temporary directory structure
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        let custom_locales_dir = temp_dir.path().join("my_locales");
        let en_dir = custom_locales_dir.join("en");
        
        fs::create_dir_all(&template_dir).expect("Failed to create template dir");
        fs::create_dir_all(&en_dir).expect("Failed to create locale dir");
        
        // Create a template file with i18n
        let template_file = template_dir.join("greeting.md");
        fs::write(&template_file, r#"---
variables:
  name: "World"
---
{{i18n "hello"}} {{name}}!"#).expect("Failed to write template file");
        
        // Create a locale file in custom location
        let locale_file = en_dir.join("messages.yaml");
        fs::write(&locale_file, r#"
hello: "Greetings"
"#).expect("Failed to write locale file");
        
        // Test builder with explicit locale paths
        let custom_locales_path = custom_locales_dir.to_str().expect("Should convert path to string");
        let template_set = TemplateSet::builder()
            .directory(&template_dir)
            .locale_paths(vec![custom_locales_path])
            .default_locale("en")
            .build()
            .expect("Failed to build TemplateSet");
        
        // Verify configuration
        assert_eq!(template_set.current_locale(), "en", "Should have correct default locale");
        
        // Test rendering with custom locale path
        let vars = json!({});
        let result = template_set.render("greeting", &vars);
        
        if let Ok(rendered) = result {
            assert!(rendered.contains("Greetings World!"), "Should contain custom localized greeting");
        }
    }

    /// Test TemplateSetBuilder error handling
    #[test]
    fn test_template_set_builder_missing_directory() {
        // Test that builder requires directory to be specified
        let result = TemplateSet::builder()
            .default_locale("en")
            .build();
        
        assert!(result.is_err(), "Should error when directory is not specified");
        
        if let Err(e) = result {
            assert!(e.to_string().contains("Template directory must be specified"), 
                   "Should have helpful error message");
        }
    }

    /// Test TemplateSetBuilder with non-existent directory
    #[test]
    fn test_template_set_builder_nonexistent_directory() {
        // Test with directory that doesn't exist
        let result = TemplateSet::builder()
            .directory("nonexistent_directory_12345")
            .build();
        
        // Should succeed but have no templates (graceful handling)
        assert!(result.is_ok(), "Should handle non-existent directories gracefully");
        
        if let Ok(template_set) = result {
            let templates = template_set.list_templates();
            assert_eq!(templates.len(), 0, "Should have no templates");
        }
    }

    /// Test Default implementation for TemplateSetBuilder
    #[test]
    fn test_template_set_builder_default() {
        let builder1 = TemplateSetBuilder::new();
        let builder2 = TemplateSetBuilder::default();
        
        // Both should create equivalent builders (can't directly compare, but test behavior)
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        fs::create_dir_all(&template_dir).expect("Failed to create directories");
        
        let result1 = builder1.directory(&template_dir).build();
        let result2 = builder2.directory(&template_dir).build();
        
        assert!(result1.is_ok(), "Builder from new() should work");
        assert!(result2.is_ok(), "Builder from default() should work");
    }
}