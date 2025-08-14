
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("URL parsing failed: {0}")]
    Url(#[from] url::ParseError),
    
    #[error("Provider error: {code} - {message}")]
    Provider { code: String, message: String },
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Model refused to respond: {0}")]
    Refusal(String),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Failed to read prompt file: {path} - {source}")]
    PromptFileRead { 
        path: String, 
        #[source] source: std::io::Error 
    },
    
    #[error("Template variable not found: {name}")]
    TemplateVariableNotFound { name: String },
    
    #[error("Template parsing failed: {0}")]
    TemplateParsing(String),
    
    #[error("Required template variables missing: {variables:?}")]
    RequiredVariablesMissing { variables: Vec<String> },
    
    #[error("Locale not found: {locale}")]
    LocaleNotFound { locale: String },
    
    #[error("i18n key not found: {key} in locale {locale}")]
    I18nKeyNotFound { key: String, locale: String },
    
    #[error("Task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    
    #[error("Function call parameter parsing failed for '{function_name}': {source}")]
    FunctionParameterParsing { 
        function_name: String, 
        #[source] source: serde_json::Error 
    },
    
    #[error("Function '{function_name}' expected arguments but received none")]
    FunctionMissingArguments { function_name: String },
    
    #[error("Function call validation failed: {reason}")]
    FunctionValidation { reason: String },
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Config(err.to_string())
    }
}