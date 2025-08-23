
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
    
    #[error("Template variable not found: {name}\nHelp: Define this variable using .var(\"{name}\", value) or ensure it's included in your template variables.")]
    TemplateVariableNotFound { name: String },
    
    #[error("Template parsing failed: {0}\nHelp: Check your template syntax for proper Handlebars formatting and valid YAML frontmatter.")]
    TemplateParsing(String),
    
    #[error("Required template variables missing: {variables:?}\nHelp: Define these variables using .var(name, value) before rendering the template.")]
    RequiredVariablesMissing { variables: Vec<String> },
    
    #[error("Locale not found: {locale}\nHelp: Ensure the locale directory exists at locales/{locale}/ with the required YAML files.")]
    LocaleNotFound { locale: String },
    
    #[error("i18n key not found: {key} in locale {locale}{}\nHelp: Add the key '{key}' to your locale file at locales/{locale}/*.yaml{}", 
        template_file.as_ref().map(|f| format!(" (in template: {})", f)).unwrap_or_default(),
        available_keys.as_ref().map(|keys| format!("\nAvailable keys: [{}]", keys.join(", "))).unwrap_or_default()
    )]
    I18nKeyNotFound { 
        key: String, 
        locale: String,
        template_file: Option<String>,
        available_keys: Option<Vec<String>>,
    },
    
    #[error("Task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    
    #[error("Function call parameter parsing failed for '{function_name}': {source}\nHelp: Ensure the function call arguments match the expected parameter types. Check the function definition for required parameters.")]
    FunctionParameterParsing { 
        function_name: String, 
        #[source] source: serde_json::Error 
    },
    
    #[error("Function '{function_name}' expected arguments but received none\nHelp: This function requires parameters. Check the function definition and provide the required arguments.")]
    FunctionMissingArguments { function_name: String },
    
    #[error("Function call validation failed: {reason}\nHelp: Review the function parameters and ensure they meet the validation requirements.")]
    FunctionValidation { reason: String },
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Config(err.to_string())
    }
}