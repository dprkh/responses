//! Integration tests for OpenAI provider functionality.

#[cfg(test)]
mod openai_provider_tests {
    use responses::{openai, Client, OpenAIConfig, OpenAIProvider, Provider};
    use responses::provider::ProviderBuilder;
    
    #[test]
    fn test_openai_config_creation() {
        let config = OpenAIConfig {
            api_key: "sk-test-key".to_string(),
        };
        
        assert_eq!(config.api_key, "sk-test-key");
    }
    
    #[test] 
    fn test_openai_provider_creation() {
        let config = OpenAIConfig {
            api_key: "sk-test-key".to_string(),
        };
        
        let provider = OpenAIProvider::new(config).unwrap();
        assert_eq!(provider.name(), "openai");
    }
    
    #[test]
    fn test_openai_builder_with_api_key() {
        let provider = openai()
            .api_key("sk-test-key")
            .build()
            .unwrap();
            
        assert_eq!(provider.name(), "openai");
    }
    
    #[test]
    fn test_openai_builder_with_config() {
        let config = OpenAIConfig {
            api_key: "sk-test-key".to_string(),
        };
        
        let provider = openai()
            .with_config(config)
            .build()
            .unwrap();
            
        assert_eq!(provider.name(), "openai");
    }
    
    #[test]
    fn test_openai_builder_fails_without_config() {
        let result = openai().build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("OpenAI configuration not provided"));
    }
    
    #[test]
    fn test_openai_client_creation() {
        let provider = openai()
            .api_key("sk-test-key")
            .build()
            .unwrap();
            
        let client = Client::new(provider);
        // Basic smoke test - if we get here, the client was created successfully
        let _ = client.text().model("gpt-4");
    }
    
    // Environment variable test - will only pass if OPENAI_API_KEY is set
    #[test] 
    fn test_openai_from_env_requires_api_key() {
        // This test will pass if OPENAI_API_KEY is set, otherwise it should fail with helpful error
        match std::env::var("OPENAI_API_KEY") {
            Ok(_) => {
                // If the env var exists, test should pass
                let result = openai().from_env();
                assert!(result.is_ok());
                
                let provider = result.unwrap().build().unwrap();
                assert_eq!(provider.name(), "openai");
            }
            Err(_) => {
                // If env var doesn't exist, should get config error
                let result = openai().from_env();
                assert!(result.is_err());
                assert!(result.unwrap_err().to_string().contains("OPENAI_API_KEY environment variable not set"));
            }
        }
    }
    
    #[test]
    fn test_openai_config_from_env_requires_api_key() {
        match std::env::var("OPENAI_API_KEY") {
            Ok(key) => {
                let config = OpenAIConfig::from_env().unwrap();
                assert_eq!(config.api_key, key);
            }
            Err(_) => {
                let result = OpenAIConfig::from_env();
                assert!(result.is_err());
                assert!(result.unwrap_err().to_string().contains("OPENAI_API_KEY environment variable not set"));
            }
        }
    }
}