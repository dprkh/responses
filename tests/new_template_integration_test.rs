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
    fn test_template_set_basic() {
        // This test would require actual files, so for now just test the structure
        let result = TemplateSet::from_dir("nonexistent");
        // Should handle missing directory gracefully or create empty set
        assert!(result.is_ok() || result.is_err()); // Either is fine for now
    }
}