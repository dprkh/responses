//! Basic integration test to verify the essential functionality works.

#[cfg(test)]
mod basic_integration_tests {
    use responses::prompt::template::PromptTemplate;
    
    #[test]
    fn loads_simple_template() {
        let content = r#"---
variables:
  role: "assistant"
---

You are a {{role}}."#;

        let template = PromptTemplate::from_content(content).unwrap();
        let result = template.render(&serde_json::json!({})).unwrap();
        
        assert!(result.contains("You are a assistant"));
    }

    #[test]
    fn handles_template_without_frontmatter() {
        let content = "Simple template content.";
        let template = PromptTemplate::from_content(content).unwrap();
        let result = template.render(&serde_json::json!({})).unwrap();
        
        assert_eq!(result, "Simple template content.");
    }

    #[test]
    fn substitutes_runtime_variables() {
        let content = "Hello {{name}}, you are {{age}} years old.";
        let template = PromptTemplate::from_content(content).unwrap();
        
        let vars = serde_json::json!({
            "name": "Alice",
            "age": 30
        });
            
        let result = template.render(&vars).unwrap();
        assert_eq!(result, "Hello Alice, you are 30 years old.");
    }
}