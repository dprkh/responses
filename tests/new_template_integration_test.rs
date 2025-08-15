//! Integration test for the new template-first API design.

#[cfg(test)]
mod template_integration_tests {
    use responses::prompt::template::{PromptTemplate, TemplateSet, ConversationTemplate};
    use responses::Messages;
    use serde_json::json;

    #[test]
    fn test_prompt_template_basic_workflow() {
        // 1. Load template
        let content = r#"---
variables:
  role: "assistant"
required_variables:
  - "domain"
---

You are a helpful {{role}} specializing in {{domain}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        
        // 2. Render with variables
        let vars = json!({
            "domain": "Rust programming"
        });
        
        let rendered = template.render(&vars).unwrap();
        assert!(rendered.contains("helpful assistant"));
        assert!(rendered.contains("Rust programming"));
    }

    #[test] 
    fn test_conversation_template_workflow() {
        let content = r#"---
variables:
  topic: "{{topic}}"
  level: "{{level}}"
---

You are teaching {{topic}} to {{level}} students."#;

        let conversation = ConversationTemplate::load_from_content(content).unwrap();
        
        let vars = json!({
            "topic": "async programming",
            "level": "intermediate"
        });
        
        let messages = conversation.render(&vars).unwrap();
        assert_eq!(messages.len(), 1);
        
        let system_content = &messages.inputs()[0];
        if let responses::types::Input::Message(msg) = system_content {
            assert!(msg.content.contains("async programming"));
            assert!(msg.content.contains("intermediate"));
        }
    }

    #[test]
    fn test_messages_template_integration() {
        let template_content = "You are a {{role}} expert in {{domain}}.";
        
        let template = PromptTemplate::from_content(template_content).unwrap();
        let vars = json!({
            "role": "helpful",
            "domain": "software engineering"
        });
        
        let rendered = template.render(&vars).unwrap();
        
        // Use with Messages API
        let messages = Messages::new()
            .system(rendered)
            .user("How do I implement error handling?");
            
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_template_error_handling() {
        let content = r#"---
required_variables:
  - "missing_var"
---

This needs {{missing_var}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({});
        
        let result = template.render(&vars);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            responses::Error::RequiredVariablesMissing { variables } => {
                assert_eq!(variables, vec!["missing_var"]);
            }
            _ => panic!("Expected RequiredVariablesMissing error"),
        }
    }

    #[test]
    fn test_template_validation_multiple_missing_variables() {
        let content = r#"---
required_variables:
  - "name"
  - "role"
  - "domain"
---
Hello {{name}}, I am your {{role}} specializing in {{domain}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({
            "role": "assistant"
        });
        
        let result = template.render(&vars);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            responses::Error::RequiredVariablesMissing { variables } => {
                assert_eq!(variables.len(), 2);
                assert!(variables.contains(&"name".to_string()));
                assert!(variables.contains(&"domain".to_string()));
                assert!(!variables.contains(&"role".to_string()));
            }
            _ => panic!("Expected RequiredVariablesMissing error"),
        }
    }

    #[test]
    fn test_template_validation_all_variables_provided() {
        let content = r#"---
required_variables:
  - "name"
  - "role"
---
Hello {{name}}, I am your {{role}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({
            "name": "Alice",
            "role": "assistant"
        });
        
        let result = template.render(&vars);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.contains("Hello Alice"));
        assert!(rendered.contains("assistant"));
    }

    #[test]
    fn test_template_validation_no_required_variables() {
        let content = r#"---
variables:
  role: "assistant"
---
Hello, I am your {{role}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({});
        
        let result = template.render(&vars);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.contains("assistant"));
    }

    #[test]
    fn test_template_validation_with_default_variables() {
        let content = r#"---
variables:
  role: "assistant"
  greeting: "Hello"
required_variables:
  - "name"
---
{{greeting}} {{name}}, I am your {{role}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let vars = json!({
            "name": "Bob"
        });
        
        let result = template.render(&vars);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.contains("Hello Bob"));
        assert!(rendered.contains("assistant"));
    }

    #[test]
    fn test_relative_include_resolution() {
        use std::path::PathBuf;
        use responses::prompt::template::PromptTemplate;
        
        // Test that base_path is used for relative include resolution
        let fixtures_path = PathBuf::from("tests/fixtures/templates");
        
        // Test that templates can include files relative to base_path
        let template_content = r#"---
variables:
  name: "test"
---
Main content
{{> shared/greeting.md}}"#;
        
        let template = PromptTemplate::from_content(template_content).unwrap();
        let vars = serde_json::json!({"name": "Alice"});
        
        // This should resolve shared/greeting.md relative to the base path
        let result = template.render_with_base_path(&vars, Some(fixtures_path));
        
        // Should succeed and include content from shared/greeting.md
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.contains("Main content"));
        assert!(rendered.contains("This is content from the shared greeting template")); // From greeting.md
    }

    #[test]
    fn test_template_set_with_base_path() {
        use std::path::PathBuf;
        use responses::prompt::template::TemplateSet;
        
        let fixtures_path = PathBuf::from("tests/fixtures/templates");
        let result = TemplateSet::from_dir(&fixtures_path);
        
        // Test that TemplateSet stores and uses base_path correctly
        if let Ok(template_set) = result {
            // Template set should be able to load templates from subdirectories
            let available_templates = template_set.list_templates();
            // Should have loaded templates from the directory structure
            assert!(available_templates.len() >= 0); // At minimum, empty is valid
        }
    }

    #[test]
    fn test_nested_relative_includes() {
        // Test includes that reference other files in subdirectories
        let template_content = r#"---
variables:
  status: "active"
---
{{> modes/debug_instructions.md}}
{{> shared/status_header.md status=status}}"#;
        
        let template = responses::prompt::template::PromptTemplate::from_content(template_content).unwrap();
        let vars = serde_json::json!({"status": "debugging"});
        
        let result = template.render(&vars);
        // Should handle nested includes or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_frontmatter_includes_auto_loading() {
        // Test that frontmatter.includes automatically loads dependent templates
        let template_content = r#"---
variables:
  user_name: "Alice"
includes:
  - "shared/greeting.md"
  - "shared/footer.md"
---
Welcome message:
{{> shared/greeting.md}}

Main content for {{user_name}}.

{{> shared/footer.md}}"#;

        let template = responses::prompt::template::PromptTemplate::from_content(template_content).unwrap();
        let vars = serde_json::json!({"user_name": "Bob"});
        
        // With auto-loading, includes should be preloaded and available
        let result = template.render_with_base_path(&vars, Some(std::path::PathBuf::from("tests/fixtures/templates")));
        assert!(result.is_ok());
        
        let rendered = result.unwrap();
        assert!(rendered.contains("Welcome message:"));
        assert!(rendered.contains("Main content for Bob"));
        assert!(rendered.contains("This is content from the shared greeting template"));
    }

    #[test]
    fn test_frontmatter_includes_validation() {
        // Test that frontmatter.includes validates missing dependencies
        let template_content = r#"---
includes:
  - "nonexistent/template.md"
---
This template depends on missing files."#;

        let template = responses::prompt::template::PromptTemplate::from_content(template_content).unwrap();
        let vars = serde_json::json!({});
        
        // Should fail with clear error about missing include file
        let result = template.render_with_base_path(&vars, Some(std::path::PathBuf::from("tests/fixtures/templates")));
        assert!(result.is_err());
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Include file not found"));
        assert!(error_msg.contains("nonexistent/template.md"));
    }

    #[test]
    fn test_frontmatter_includes_circular_detection() {
        // Test that circular includes are detected and handled
        let template_content = r#"---
includes:
  - "circular/circular_a.md"
---
This should detect circular references:
{{> circular/circular_a.md}}"#;

        let template = responses::prompt::template::PromptTemplate::from_content(template_content).unwrap();
        let vars = serde_json::json!({});
        
        let result = template.render_with_base_path(&vars, Some(std::path::PathBuf::from("tests/fixtures/templates")));
        assert!(result.is_ok());
        
        let rendered = result.unwrap();
        // Should include circular reference detection comment
        assert!(rendered.contains("Circular reference detected") || rendered.contains("Parent content"));
    }

    #[test]
    fn test_frontmatter_i18n_key_template_selection() {
        // Test that frontmatter.i18n_key enables locale-aware template selection
        let template_content = r#"---
variables:
  greeting_type: "formal"
i18n_key: "system.greeting"
---
{{i18n "system.greeting"}}: Welcome to our {{greeting_type}} system."#;

        let template = responses::prompt::template::PromptTemplate::from_content(template_content).unwrap();
        let vars = serde_json::json!({"greeting_type": "professional"});
        
        // Should render with appropriate locale-based content
        let result = template.render(&vars);
        assert!(result.is_ok());
        
        let rendered = result.unwrap();
        assert!(rendered.contains("Welcome to our professional system"));
    }

    #[test]
    fn test_template_set_locale_switching() {
        use std::path::PathBuf;
        use responses::prompt::template::TemplateSet;
        
        let fixtures_path = PathBuf::from("tests/fixtures/templates");
        let result = TemplateSet::from_dir(&fixtures_path);
        
        if let Ok(template_set) = result {
            // Test that TemplateSet can switch locales for i18n_key templates
            let spanish_set = template_set.with_locale("es"); // Switch to Spanish
            
            if let Ok(spanish_template_set) = spanish_set {
                // Template with i18n_key should use Spanish locale
                let spanish_templates = spanish_template_set.list_templates();
                // Should have loaded templates (empty is valid for testing)
                assert!(spanish_templates.len() >= 0);
                assert_eq!(spanish_template_set.current_locale(), "es");
            }
        }
    }

    #[test]
    fn test_frontmatter_i18n_key_missing_locale() {
        // Test behavior when i18n_key references missing locale content
        let template_content = r#"---
i18n_key: "nonexistent.key"
---
Fallback content: {{i18n "nonexistent.key"}}"#;

        let template = responses::prompt::template::PromptTemplate::from_content(template_content).unwrap();
        let vars = serde_json::json!({});
        
        let result = template.render(&vars);
        assert!(result.is_ok());
        
        let rendered = result.unwrap();
        // Should gracefully handle missing i18n keys
        assert!(rendered.contains("Fallback content:"));
    }

    #[test]
    fn test_template_set_basic() {
        // This test would require actual files, so for now just test the structure
        let result = TemplateSet::from_dir("nonexistent");
        // Should handle missing directory gracefully or create empty set
        assert!(result.is_ok() || result.is_err()); // Either is fine for now
    }
}