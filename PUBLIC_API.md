# Responses API Documentation

Rust library for OpenAI's Responses API with fluent DSL, templates, and structured outputs. Supports both Azure OpenAI and plain OpenAI providers.

## Table of Contents

- [Getting Started](#getting-started)
- [Client Setup](#client-setup)
  - [Azure Configuration](#azure-configuration)
  - [OpenAI Configuration](#openai-configuration)
- [Core APIs](#core-apis)
  - [Text Generation](#text-generation)
  - [Structured Outputs](#structured-outputs)
  - [Enhanced Function Calling](#enhanced-function-calling)
- [Conversation Management](#conversation-management)
  - [Messages Builder](#messages-builder)
  - [Conversation Persistence](#conversation-persistence)
  - [Advanced Message Operations](#advanced-message-operations)
- [Function Calling with #[tool]](#function-calling-with-tool)
  - [Function Handlers](#function-handlers)
  - [Function Call Processing](#function-call-processing)
  - [Function Macros](#function-macros)
- [Multiline Message Support](#multiline-message-support)
- [Prompt Templates](#prompt-templates)
  - [Markdown Template System](#markdown-template-system)
  - [Template Set Management](#template-set-management)
  - [Template Validation and Dependencies](#template-validation-and-dependencies)
  - [Internationalization (i18n)](#internationalization-i18n)
  - [Template Variables and Composition](#template-variables-and-composition)
  - [Conversation Templates](#conversation-templates)
- [LLM-as-a-Judge](#llm-as-a-judge)
  - [Basic Judge Usage](#basic-judge-usage)
  - [Template-Based Evaluation](#template-based-evaluation)
  - [Test Assertions](#test-assertions)
- [Temperature Control](#temperature-control)
- [Error Handling](#error-handling)
- [Examples](#examples)

## Getting Started

Add this to your `Cargo.toml`:

```toml
[dependencies]
responses = "0.2.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["rt", "macros"] }

# Note: schemars is re-exported by responses to prevent version conflicts
# Use responses::schemars instead of adding a separate schemars dependency
```

Environment variables:

```bash
# For Azure OpenAI
export AZURE_OPENAI_API_KEY="your-api-key"
export AZURE_OPENAI_RESOURCE="your-resource-name"
export AZURE_OPENAI_API_VERSION="2025-03-01-preview"

# For OpenAI
export OPENAI_API_KEY="sk-your-api-key-here"

# Optional
export RESPONSES_LOCALES_PATH="custom/locales/path,another/path"
```

## Client Setup

### Azure Configuration

```rust
use responses::{azure, Client};
use responses::provider::ProviderBuilder;

#[tokio::main]
async fn main() -> responses::Result<()> {
    let provider = azure()
        .from_env()?
        .build()?;
    let client = Client::new(provider);
    
    // Use client for requests...
    Ok(())
}
```

**Client API:**
```rust
impl<P: Provider> Client<P> {
    pub fn new(provider: P) -> Self;
    pub fn provider(&self) -> &P;
    pub fn text(&self) -> TextRequestBuilder<'_, P>;
    pub fn structured<T>(&self) -> StructuredRequestBuilder<'_, P, T>;
    pub fn structured_with_name<T>(&self, name: String) -> StructuredRequestBuilder<'_, P, T>;
}
```

**Configuration options:**
```rust
// From environment
let provider = azure().from_env()?.build()?;

// Manual config  
let config = AzureConfig {
    api_key: "key".to_string(),
    resource: "resource".to_string(), 
    api_version: "2025-03-01-preview".to_string(),
};
let provider = azure().with_config(config).build()?;

// Builder pattern
let provider = azure()
    .api_key("key")
    .resource("resource")
    .api_version("2025-03-01-preview")
    .build()?;
```

### OpenAI Configuration

```rust
use responses::{openai, Client};
use responses::provider::ProviderBuilder;

#[tokio::main]
async fn main() -> responses::Result<()> {
    let provider = openai()
        .from_env()?
        .build()?;
    let client = Client::new(provider);
    
    // Use client for requests...
    Ok(())
}
```

**OpenAI Configuration options:**
```rust
// From environment
let provider = openai().from_env()?.build()?;

// Manual config  
let config = OpenAIConfig {
    api_key: "sk-your-api-key-here".to_string(),
};
let provider = openai().with_config(config).build()?;

// Builder pattern
let provider = openai()
    .api_key("sk-your-api-key-here")
    .build()?;

## Core APIs

### Text Generation

```rust
use responses::{azure, Client};
use responses::provider::ProviderBuilder;

let provider = azure().from_env()?.build()?;
let client = Client::new(provider);

// Or with OpenAI
let provider = openai().from_env()?.build()?;
let client = Client::new(provider);

let response = client
    .text()
    .model("gpt-4o")
    .system("You are a helpful assistant")
    .user("Tell me a joke")
    .send()
    .await?;

if let Some(Ok(text)) = response.message {
    println!("Response: {}", text);
}
```

**TextRequestBuilder API:**
```rust
impl<P: Provider> TextRequestBuilder<P> {
    // Basic message methods
    pub fn model<S: Into<String>>(self, model: S) -> Self;
    pub fn with_model(self, model: &'static str) -> Self;
    pub fn system<S: Into<String>>(self, content: S) -> Self;
    pub fn user<S: Into<String>>(self, content: S) -> Self;
    pub fn assistant<S: Into<String>>(self, content: S) -> Self;
    pub fn developer<S: Into<String>>(self, content: S) -> Self;
    
    // Template integration
    pub fn system_from_md<P: AsRef<Path>>(self, path: P) -> Result<Self>;
    pub fn assistant_from_md<P: AsRef<Path>>(self, path: P) -> Result<Self>;
    pub fn var<K: Into<String>, V: Serialize>(self, key: K, value: V) -> Self;
    pub fn with_locale<S: Into<String>>(self, locale: S, locale_paths: &[&str]) -> Result<Self>;
    
    // Tools and configuration
    pub fn tools(self, tools: Vec<Tool>) -> Self;
    pub fn tool_choice(self, choice: ToolChoice) -> Self;
    pub fn safety_identifier<S: Into<String>>(self, id: S) -> Self;
    pub fn temperature(self, temp: f32) -> Self;
    
    // Conversation management
    pub fn messages(self, messages: Messages) -> Self;
    pub fn continue_conversation(self, messages: &Messages) -> Self;
    pub fn from_messages(self, messages: Vec<InputMessage>) -> Self;
    pub fn add_messages<I, S>(self, messages: I) -> Self 
    where I: IntoIterator<Item = (Role, S)>, S: Into<String>;
    
    // Execution
    pub async fn send(self) -> Result<Response<String>>;
}
```

**Usage:**
```rust
let response = client
    .text()
    .model("gpt-4o")
    .system("You are a helpful assistant")
    .user("Tell me a joke")
    .temperature(0.7)
    .send()
    .await?;
```

### Structured Outputs

Type-safe JSON outputs using `responses::schemars::JsonSchema`.

```rust
use responses::schemars::JsonSchema;  // Use re-exported version
use serde::Deserialize;

#[derive(JsonSchema, Deserialize)]
struct WeatherResponse {
    temperature: f64,
    condition: String,
    humidity: i32,
}

let weather = client
    .structured::<WeatherResponse>()
    .model("gpt-4o")
    .system("Provide weather information")
    .user("Weather in San Francisco?")
    .send()
    .await?;

**StructuredRequestBuilder API:**
```rust
impl<P: Provider, T> StructuredRequestBuilder<P, T> 
where T: JsonSchema + for<'a> Deserialize<'a>
{
    // Basic message methods
    pub fn model<S: Into<String>>(self, model: S) -> Self;
    pub fn system<S: Into<String>>(self, content: S) -> Self;
    pub fn user<S: Into<String>>(self, content: S) -> Self;
    pub fn assistant<S: Into<String>>(self, content: S) -> Self;
    pub fn developer<S: Into<String>>(self, content: S) -> Self;
    
    // Template integration
    pub fn system_from_md<P: AsRef<Path>>(self, path: P) -> Result<Self>;
    pub fn assistant_from_md<P: AsRef<Path>>(self, path: P) -> Result<Self>;
    pub fn var<K: Into<String>, V: Serialize>(self, key: K, value: V) -> Self;
    pub fn with_locale<S: Into<String>>(self, locale: S, locale_paths: &[&str]) -> Result<Self>;
    
    // Tools and configuration
    pub fn tools(self, tools: Vec<Tool>) -> Self;
    pub fn tool_choice(self, choice: ToolChoice) -> Self;
    pub fn safety_identifier<S: Into<String>>(self, id: S) -> Self;
    pub fn temperature(self, temp: f32) -> Self;
    
    // Conversation management
    pub fn messages(self, messages: Messages) -> Self;
    pub fn continue_conversation(self, messages: &Messages) -> Self;
    pub fn from_messages(self, messages: Vec<InputMessage>) -> Self;
    pub fn add_messages<I, S>(self, messages: I) -> Self 
    where I: IntoIterator<Item = (Role, S)>, S: Into<String>;
    
    // Execution
    pub async fn send(self) -> Result<Response<T>>;
}
```

**Custom schema names:**
```rust
let response = client
    .structured_with_name::<WeatherResponse>("DetailedWeather".to_string())
    .model("gpt-4o")
    .user("Get weather for New York")
    .send()
    .await?;
```

### Enhanced Function Calling

Auto-deserializing function calls with built-in schema generation, type-safe execution, and automatic parameter parsing.

```rust
use responses::{tool, types::ToolChoice};

#[tool]
async fn get_weather(city: String, units: Option<String>) -> Result<WeatherResponse> {
    Ok(WeatherResponse { temperature: 22.0, condition: "Sunny".to_string() })
}

let response = client
    .text()
    .model("gpt-4o")
    .user("Weather in Paris?")
    .tools(vec![get_weather_handler().into()])  // Auto-generated handler
    .tool_choice(ToolChoice::Auto)
    .send()
    .await?;
```

The `#[tool]` macro generates `{FunctionName}Params` struct, `{FunctionName}Handler`, and `{function_name}_handler()` function.




## Conversation Management

### Messages Builder

```rust
use responses::{Messages, messages};

// Builder pattern
let conversation = Messages::new()
    .system("You are a helpful assistant")
    .user("Hello!")
    .assistant("Hi there!")
    .user("What can you help with?");

// Macro syntax  
let conversation = messages! {
    system: "You are a coding expert",
    user: "Explain recursion",
    assistant: "Recursion is when a function calls itself..."
};
```

#### `Messages` API

```rust
impl Messages {
    pub fn new() -> Self;
    
    // Message builders
    pub fn system<S: Into<String>>(self, content: S) -> Self;
    pub fn user<S: Into<String>>(self, content: S) -> Self;
    pub fn assistant<S: Into<String>>(self, content: S) -> Self;
    pub fn developer<S: Into<String>>(self, content: S) -> Self;
    pub fn add_message<S: Into<String>>(self, role: Role, content: S) -> Self;
    
    // Note: Previously had system_prompt(), user_message(), assistant_response(), developer_note()
    // These have been removed in favor of the simpler system(), user(), assistant(), developer() methods
    
    // Bulk operations
    pub fn add_messages<I, S>(self, messages: I) -> Self 
    where I: IntoIterator<Item = (Role, S)>, S: Into<String>;
    pub fn extend(self, other: Messages) -> Self;
    pub fn extend_inputs(self, inputs: Vec<Input>) -> Self;
    
    // Constructors
    pub fn from_inputs(inputs: Vec<Input>) -> Self;
    pub fn from_messages(messages: Vec<InputMessage>) -> Self;
    pub fn from_conversation_template<P: AsRef<std::path::Path>>(path: P, vars: &serde_json::Value) -> Result<Self>;
    
    // Accessors
    pub fn into_inputs(self) -> Vec<Input>;
    pub fn inputs(&self) -> &[Input];
    pub fn render_inputs(&self) -> Vec<Input>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn first(&self) -> Option<&Input>;
    pub fn last(&self) -> Option<&Input>;
    
    // Slicing and filtering
    pub fn take_last(&self, n: usize) -> Messages;
    pub fn take_first(&self, n: usize) -> Messages;
    pub fn pop_last(self, n: usize) -> Self;
    pub fn filter_by_role(&self, role: Role) -> Messages;
    
    // Templates
    pub fn as_template(&self) -> Messages;
}
```

### Conversation Persistence

```rust
// Reusable template
let template = Messages::new().system("You are a helpful tutor");

// Use multiple times
let algebra_convo = template.clone().user("Explain linear equations");
let geometry_convo = template.clone().user("Explain Pythagorean theorem");

// Continue conversation (borrows vs consumes)
client.text().messages(conversation).send().await?;         // Consumes
client.text().continue_conversation(&conversation).send().await?;  // Borrows
```

### Advanced Message Operations

```rust
// Slicing operations
let recent = full_conversation.take_last(2);
let user_only = full_conversation.filter_by_role(Role::User);
let initial = full_conversation.take_first(3);

// Bulk operations
let bulk = vec![(Role::System, "prompt"), (Role::User, "hello")];
let conversation = Messages::new().add_messages(bulk);
```


## Multiline Message Support

All message methods seamlessly support multiline strings, making it easy to create complex prompts.

### System Prompts

```rust
let response = client
    .text()
    .model("gpt-4o")
    .system(r#"You are a helpful coding assistant with expertise in multiple languages.

Your responses should:
- Be clear and concise
- Include working code examples  
- Explain complex concepts step by step
- Provide best practices when relevant

Always format code blocks properly and explain assumptions."#)
    .user("How do I implement a binary search in Rust?")
    .send()
    .await?;
```

### Complex Conversations

```rust
let conversation = Messages::new()
    .system(r#"You are a senior software architect.

Guidelines:
- Consider scalability and performance
- Suggest appropriate technologies
- Ask clarifying questions when needed
- Provide concrete examples"#)
    .user(r#"I need to design a chat system that handles:
- 10,000 concurrent users
- Message history storage
- File sharing
- Group chats up to 100 members

What architecture would you recommend?"#)
    .assistant(r#"For this scale, I'd recommend:

**Core Components:**
1. WebSocket Gateway (Node.js/Go)
2. Message Service (microservice)  
3. Redis Cluster for caching
4. PostgreSQL for persistence
5. File Storage Service (S3/MinIO)

**Key Patterns:**
- Event-driven architecture
- Database sharding by room ID
- CDN for file distribution
- Horizontal scaling

Would you like details on any component?"#);
```

### Code Review Example

```rust
let code_review = Messages::new()
    .system("You are an experienced code reviewer focusing on quality, performance, security, and best practices.")
    .user(r#"Please review this function:

```python
def process_users(data):
    users = []
    for item in data:
        if item['age'] > 18:
            user = {
                'name': item['name'].strip(),
                'email': item['email'].lower(),
                'age': item['age']
            }
            users.append(user)
    return users
```"#);
```

## Prompt Templates

The prompt template system enables you to separate prompt engineering from code by storing prompts in external Markdown files with support for variables, internationalization, and template composition.

### Markdown Template System

Store your prompts in `.md` files with YAML frontmatter for configuration, variables, validation, and dependency management.

#### Enhanced Template Structure

```markdown
---
variables:
  role: "{{role}}"
  domain: "{{domain}}"
  experience_years: "{{experience_years}}"
required_variables:
  - "role"
  - "domain"
includes:
  - "shared/header.md"
  - "shared/footer.md"
i18n_key: "coding_assistant"
---

{{> shared/header.md}}

# {{i18n "system.title"}}

{{i18n "system.intro" role=role}}

## Expertise
You specialize in {{domain}} with {{experience_years}} years of experience.

## Guidelines
{{#i18n "guidelines.list"}}
- {{i18n "guidelines.be_helpful"}}
- {{i18n "guidelines.provide_examples"}}
- {{i18n "guidelines.ask_clarification"}}
{{/i18n}}

{{> shared/footer.md}}
```

#### Frontmatter Configuration

```yaml
---
# Default variables (can be overridden at render time)
variables:
  greeting: "Hello"
  role: "assistant"
  
# Required variables (must be provided at render time)
required_variables:
  - "user_name"
  - "task_type"
  
# Template dependencies (validated when base_path is provided)
includes:
  - "shared/header.md"
  - "components/task_instructions.md"
  - "shared/footer.md"
  
# Internationalization key for locale-aware template selection
i18n_key: "system.main_prompt"
---
```

#### Loading Templates in Code

```rust
use responses::{azure, Messages, Client};
use responses::prompt::PromptTemplate;

let provider = azure().from_env()?.build()?;
let client = Client::new(provider);

// Works with OpenAI too
// let provider = openai().from_env()?.build()?;
// let client = Client::new(provider);

// Simple template loading
let response = client
    .text()
    .model("gpt-4o")
    .system_from_md("prompts/coding_assistant.md")?
    .var("role", "senior engineer")
    .var("domain", "Rust programming")
    .var("experience_years", 8)
    .user("How do I implement async streams?")
    .send()
    .await?;
```

#### Enhanced Fluent API - Any Order Chaining

The fluent API supports **any-order chaining** for variables, locale, and template loading. Templates are rendered lazily when `send()` is called, allowing maximum flexibility.

```rust
// All of these work identically - variables and locale are applied when send() is called

// Variables first, then template
let response = client
    .text()
    .model("gpt-4o")
    .var("role", "senior engineer")
    .var("domain", "Rust programming")
    .with_locale("en", &["templates/locales"])?
    .system_from_md("prompts/coding_assistant.md")?
    .user("How do I implement async streams?")
    .send()
    .await?;

// Template first, then variables (the original way)
let response = client
    .text()
    .model("gpt-4o")
    .system_from_md("prompts/coding_assistant.md")?
    .var("role", "senior engineer")
    .var("domain", "Rust programming")
    .with_locale("en", &["templates/locales"])?
    .user("How do I implement async streams?")
    .send()
    .await?;

// Mixed order - completely flexible!
let response = client
    .text()
    .model("gpt-4o")
    .var("role", "senior engineer")
    .system_from_md("prompts/coding_assistant.md")?
    .with_locale("en", &["templates/locales"])?
    .var("domain", "Rust programming")
    .user("How do I implement async streams?")
    .send()
    .await?;
```

#### How It Works

1. **Lazy Evaluation**: Templates are stored (not rendered) when `*_from_md()` is called
2. **Variable Accumulation**: Variables added with `.var()` are accumulated in a map
3. **Locale Tracking**: The current locale is tracked for i18n rendering
4. **Render on Send**: When `send()` is called, all templates are rendered with accumulated variables and locale
5. **Error Handling**: Missing variables or i18n keys are caught during the final render phase

This enables maximum flexibility while maintaining type safety and error handling.

#### Complete Template API

```rust
impl PromptTemplate {
    // === LOADING AND CREATION ===
    
    /// Load template from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self>;
    
    /// Create template from string content
    pub fn from_content(content: &str) -> Result<Self>;
    
    // === LOCALE SUPPORT ===
    
    /// Set locale for internationalization
    pub fn with_locale(mut self, locale: &str, locale_paths: &[&str]) -> Result<Self>;
    
    // === FRONTMATTER ACCESS ===
    
    /// Get list of required variables from frontmatter
    pub fn required_variables(&self) -> &[String];
    
    /// Get list of include dependencies from frontmatter
    pub fn includes(&self) -> Vec<String>;
    
    // === VALIDATION ===
    
    /// Validate that all required variables are provided
    pub fn validate_variables(&self, vars: &serde_json::Value) -> Result<()>;
    
    /// Validate that all frontmatter includes exist at base path
    pub fn validate_includes(&self, base_path: &Path) -> Result<()>;
    
    // === RENDERING ===
    
    /// Render template with variables
    pub fn render(&self, vars: &serde_json::Value) -> Result<String>;
    
    /// Render template with base path for relative includes
    pub fn render_with_base_path(&self, vars: &serde_json::Value, base_path: Option<PathBuf>) -> Result<String>;
    
    /// Render with serializable context object
    pub fn render_with_context<T: Serialize>(&self, context: &T) -> Result<String>;
    
    // === FLUENT API (Variable Chaining) ===
    
    /// Add variable to template (fluent API)
    pub fn var<K: Into<String>, V: serde::Serialize>(mut self, key: K, value: V) -> Self;
    
    /// Render with accumulated variables
    pub fn render_with_vars(&self) -> Result<String>;
    
    /// Render with accumulated + additional variables
    pub fn render_with_additional_vars(&self, additional_vars: &serde_json::Value) -> Result<String>;
}
```

#### Usage Examples

```rust
use responses::prompt::PromptTemplate;
use std::path::PathBuf;

// === BASIC LOADING AND RENDERING ===

let template = PromptTemplate::load("prompts/system.md")?;
let vars = serde_json::json!({
    "role": "assistant",
    "domain": "machine learning",
    "user_name": "Alice"
});
let rendered = template.render(&vars)?;

// === FLUENT API WITH VARIABLE CHAINING ===

let response = client
    .text()
    .model("gpt-4o")
    .system_from_md("prompts/coding_assistant.md")?
    .var("role", "senior engineer")
    .var("domain", "Rust programming")
    .var("experience_years", 8)
    .user("How do I implement async streams?")
    .send()
    .await?;

// === WITH BASE PATH FOR RELATIVE INCLUDES ===

let template = PromptTemplate::load("prompts/main.md")?;
let base_path = PathBuf::from("prompts");
let rendered = template.render_with_base_path(&vars, Some(base_path))?;

// === VALIDATION BEFORE RENDERING ===

let template = PromptTemplate::load("prompts/strict.md")?;

// Check what variables are required
let required = template.required_variables();
println!("Required variables: {:?}", required);

// Check template dependencies
let includes = template.includes();
println!("Template includes: {:?}", includes);

// Validate variables before rendering
let vars = serde_json::json!({"user_name": "Bob"});
template.validate_variables(&vars)?; // Will fail if required vars missing

// Validate includes exist
let base_path = PathBuf::from("templates");
template.validate_includes(&base_path)?; // Will fail if includes don't exist

// === LOCALE SUPPORT ===

let template = PromptTemplate::load("prompts/system.md")?
    .with_locale("es", &["locales"])?
    .var("role", "asistente")
    .var("domain", "programación");
```

### Template Set Management

The `TemplateSet` provides enterprise-grade template management with organized directory structures, base path resolution, automatic locale detection, and seamless i18n support.

#### TemplateSet API

```rust
impl TemplateSet {
    // === CREATION AND LOADING ===
    
    /// Load all templates from a directory with automatic locale detection
    /// 
    /// Automatically detects and configures locales if a `locales/` 
    /// subdirectory exists, eliminating manual configuration.
    pub fn from_dir<P: AsRef<Path>>(dir: P) -> Result<Self>;
    
    /// Create a builder for advanced template set configuration
    pub fn builder() -> TemplateSetBuilder;
    
    // === LOCALE MANAGEMENT ===
    
    /// Switch locale for all templates in the set
    pub fn with_locale(mut self, locale: &str, locale_paths: &[&str]) -> Result<Self>;
    
    /// Get current locale
    pub fn current_locale(&self) -> &str;
    
    // === TEMPLATE RENDERING ===
    
    /// Render a template by name (uses base_path for includes)
    pub fn render(&self, template_name: &str, vars: &serde_json::Value) -> Result<String>;
    
    /// Render a conversation template as Messages
    pub fn render_conversation(&self, name: &str, vars: &serde_json::Value) -> Result<Messages>;
    
    // === DISCOVERY ===
    
    /// List all available template names
    pub fn list_templates(&self) -> Vec<&str>;
    
    /// Check if a template exists
    pub fn template_exists(&self, name: &str) -> bool;
    
    /// List all conversation template names
    pub fn list_conversations(&self) -> Vec<&str>;
    
    /// Check if a conversation template exists
    pub fn conversation_exists(&self, name: &str) -> bool;
}
```

#### TemplateSetBuilder API

The builder pattern provides advanced control over template loading while maintaining simplicity for common use cases.

```rust
impl TemplateSetBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self;
    
    /// Set the template directory path
    pub fn directory<P: AsRef<Path>>(self, path: P) -> Self;
    
    /// Enable automatic locale detection and configuration
    /// 
    /// When enabled, automatically detects `locales/` subdirectories 
    /// and configures i18n support without manual configuration.
    pub fn auto_configure_locales(self) -> Self;
    
    /// Set the default locale (defaults to "en")
    pub fn default_locale<S: Into<String>>(self, locale: S) -> Self;
    
    /// Add explicit locale paths (alternative to auto-detection)
    pub fn locale_paths<I, S>(self, paths: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: Into<String>;
    
    /// Build the configured TemplateSet
    pub fn build(self) -> Result<TemplateSet>;
}
```

#### Template Directory Organization

```
templates/
├── system_prompts/
│   ├── coding_assistant.md
│   ├── data_analyst.md
│   └── creative_writer.md
├── conversations/
│   ├── learning_session.md
│   ├── code_review.md
│   └── debug_session.md
├── shared/
│   ├── header.md
│   ├── footer.md
│   └── guidelines.md
├── components/
│   ├── task_instructions.md
│   └── safety_guidelines.md
└── locales/
    ├── en/
    │   └── system.yaml
    ├── es/
    │   └── system.yaml
    └── ja/
        └── system.yaml
```

#### TemplateSet Usage

```rust
use responses::prompt::TemplateSet;
use std::path::PathBuf;

// === AUTOMATIC LOCALE DETECTION ===

// Simple usage - automatically detects locales/ subdirectory
let template_set = TemplateSet::from_dir("templates")?;
// If templates/locales/ exists, i18n is automatically configured!

// === ADVANCED CONFIGURATION WITH BUILDER ===

// For more control, use the builder pattern
let template_set = TemplateSet::builder()
    .directory("templates")
    .auto_configure_locales()  // Auto-detect locales/ subdirs
    .default_locale("en")
    .build()?;

// Or specify explicit locale paths
let template_set = TemplateSet::builder()
    .directory("templates") 
    .locale_paths(["templates/locales", "locales"])
    .default_locale("fr")
    .build()?;

// === TEMPLATE DISCOVERY ===

// List available templates
let templates = template_set.list_templates();
println!("Available templates: {:?}", templates);

// === RENDERING WITH BASE PATH RESOLUTION ===

// Templates can use {{> shared/header.md}} and it resolves relative to base_path
let vars = serde_json::json!({
    "user_name": "Alice",
    "role": "engineer"
});

let rendered = template_set.render("system_prompts/coding_assistant", &vars)?;

// === LOCALE SWITCHING ===

// Switch to Spanish locale for all templates (manual path specification)
let spanish_set = template_set.with_locale("es", &["templates/locales"])?;
let spanish_rendered = spanish_set.render("system_prompts/coding_assistant", &vars)?;

// === CONVERSATION TEMPLATES ===

let conversation_msgs = template_set.render_conversation("learning_session", &vars)?;

// Use with client
let response = client
    .text()
    .model("gpt-4o")
    .messages(conversation_msgs)
    .send()
    .await?;
```

### Template Validation and Dependencies

The template system provides comprehensive validation for production environments.

#### Required Variables Validation

```markdown
---
required_variables:
  - "user_name"
  - "task_type"
  - "complexity_level"
variables:
  greeting: "Hello"
  role: "assistant"
---

Hello {{user_name}},

I'm your {{role}} ready to help with {{task_type}} tasks.
The complexity level is set to {{complexity_level}}.
```

```rust
// This will fail validation if required variables are missing
let template = PromptTemplate::load("prompts/strict.md")?;
let incomplete_vars = serde_json::json!({
    "user_name": "Alice"
    // Missing: task_type, complexity_level
});

// Validation error with clear message
match template.render(&incomplete_vars) {
    Err(Error::RequiredVariablesMissing { variables }) => {
        println!("Missing required variables: {:?}", variables);
        // Output: Missing required variables: ["task_type", "complexity_level"]
    }
    _ => unreachable!()
}
```

#### Include Dependency Management

```markdown
---
includes:
  - "shared/header.md"
  - "components/task_instructions.md"
  - "shared/footer.md"
variables:
  project_name: "{{project_name}}"
---

{{> shared/header.md}}

## Project: {{project_name}}

{{> components/task_instructions.md}}

{{> shared/footer.md}}
```

```rust
// Validate template dependencies
let template = PromptTemplate::load("prompts/main.md")?;
let base_path = PathBuf::from("templates");

// This validates that all included files exist
template.validate_includes(&base_path)?;

// Or validate automatically during rendering
let vars = serde_json::json!({"project_name": "MyProject"});
let rendered = template.render_with_base_path(&vars, Some(base_path))?;
```

#### Production Validation Workflow

```rust
use responses::prompt::TemplateSet;

// Load and validate all templates in production
async fn validate_template_environment() -> Result<TemplateSet> {
    let template_set = TemplateSet::from_dir("templates")?;
    
    // Validate each template's dependencies
    for template_name in template_set.list_templates() {
        println!("Validating template: {}", template_name);
        
        // Templates are automatically validated when rendered through TemplateSet
        // The base_path is handled automatically
    }
    
    println!("✅ All templates validated successfully");
    Ok(template_set)
}

// Use in application startup
let validated_templates = validate_template_environment().await?;
```

### Internationalization (i18n)

Templates support full internationalization with locale-specific strings and formatting. Locales must be explicitly set using `.with_locale()` - there is no automatic environment variable detection.

#### LocaleManager API

The `LocaleManager` provides comprehensive locale management for i18n features:

```rust
impl LocaleManager {
    /// Create a new LocaleManager with locales directory and default locale
    pub fn new<P: AsRef<Path>>(locales_path: P, default_locale: &str) -> Result<Self>;
    
    /// Load a specific locale into the cache
    pub fn load_locale(&mut self, locale: &str) -> Result<&LocaleData>;
    
    /// Resolve a requested locale (with fallback logic)
    pub fn resolve_locale(&mut self, requested_locale: &str) -> Result<String>;
    
    /// Get cached locale data
    pub fn get_locale(&mut self, locale: &str) -> Result<&LocaleData>;
    
    /// Resolve the file path for a locale
    pub fn resolve_locale_path(&self, locale: &str) -> Result<PathBuf>;
    
    /// Check if a locale is available
    pub fn is_valid_locale(&self, locale: &str) -> bool;
    
    /// Get the number of cached locales
    pub fn cache_size(&self) -> usize;
}
```

#### LocaleData API

The `LocaleData` provides locale-specific string and formatting operations:

```rust
impl LocaleData {
    /// Get a localized string by key
    pub fn get_string(&self, key: &str) -> Option<String>;
    
    /// Get a localized string with variable interpolation
    pub fn interpolate(&self, key: &str, variables: &HashMap<String, serde_json::Value>) -> Result<String>;
    
    /// Format a number according to locale rules
    pub fn format_number(&self, number: f64) -> String;
    
    /// Format a value as a percentage
    pub fn format_percentage(&self, value: f64) -> String;
    
}
```

#### Usage Examples

```rust
use responses::prompt::{LocaleManager, LocaleData};
use std::collections::HashMap;

// Create locale manager
let mut locale_manager = LocaleManager::new("locales", "en")?;

// Load and use locale data
let locale_data = locale_manager.load_locale("es")?;
let greeting = locale_data.get_string("greeting").unwrap_or_default();

// Interpolate variables
let mut vars = HashMap::new();
vars.insert("name".to_string(), serde_json::Value::String("Alice".to_string()));
let personalized = locale_data.interpolate("greeting_with_name", &vars)?;

// Format numbers
let formatted_number = locale_data.format_number(1234.56);
let percentage = locale_data.format_percentage(0.856);

```

#### Locale File Structure

```
prompts/
├── locales/
│   ├── en/
│   │   ├── system.yaml
│   │   └── conversations.yaml
│   ├── es/
│   │   └── system.yaml
│   └── ja/
│       └── system.yaml
└── templates/
    ├── coding_assistant.md
    └── conversations/
        └── debug_session.md
```

#### Locale Files (YAML)

```yaml
# locales/en/system.yaml
system:
  title: "System Instructions"
  intro: "You are a {role} with expertise in software development."

guidelines:
  list: "Follow these guidelines:"
  be_helpful: "Be helpful and supportive"
  provide_examples: "Provide working code examples"
  ask_clarification: "Ask for clarification when needed"

tasks:
  count:
    zero: "no tasks"
    one: "{count} task"
    other: "{count} tasks"
```

```yaml
# locales/es/system.yaml  
system:
  title: "Instrucciones del Sistema"
  intro: "Eres un {role} con experiencia en desarrollo de software."

guidelines:
  list: "Sigue estas pautas:"
  be_helpful: "Sé útil y solidario"
  provide_examples: "Proporciona ejemplos de código funcional"
  ask_clarification: "Pide aclaración cuando sea necesario"
```

#### i18n Template Features

```markdown
---
locale_key: "system"
variables:
  task_count: "{{task_count}}"
---

# {{i18n "system.title"}}

{{i18n "system.intro" role="assistant"}}

## Current Status
{{i18n "current_tasks" count=task_count tasks=task_count}}

Progress: {{format_number progress style="percent"}}

{{#if_locale "ar"}}
المحتوى بالعربية
{{/if_locale}}
```

#### Using Localized Templates

All locale setting is explicit - you must specify the desired locale:

```rust
// Set locale for request builder
let response = client
    .text()
    .model("gpt-4o")
    .system_from_md("prompts/assistant.md")?
    .with_locale("es", &["templates/locales"])?
    .var("role", "asistente")
    .var("task_count", 3)
    .send()
    .await?;

// Set locale for template directly
let template = PromptTemplate::load("prompts/system.md")?
    .with_locale("ja", &["locales"])?
    .var("role", "アシスタント");
```

### Template Variables and Composition

Templates support variables, conditionals, loops, and includes for powerful composition.

#### Variable Substitution

```rust
// Simple variables
let template = PromptTemplate::load("prompts/greeting.md")?
    .var("name", "Alice")
    .var("role", "engineer")
    .var("years_experience", 5);

// Complex objects
use serde_json::json;
let user_data = json!({
    "name": "Alice",
    "department": "Engineering",
    "skills": ["Rust", "Python", "JavaScript"]
});

let template = PromptTemplate::load("prompts/profile.md")?
    .var("user", user_data);
```

#### Template Composition

```markdown
---
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

{{> shared/footer.md locale=current_locale}}
```

#### Conditionals and Loops

```markdown
{{#if debug_mode}}
## Debug Information
Debug logging is enabled.
{{/if}}

{{#each projects}}
### {{this.name}}
Status: {{this.status}}
{{#each this.tasks}}
- [ ] {{this}}
{{/each}}
{{/each}}

{{#switch user_level}}
  {{#case "beginner"}}
Start with basic concepts.
  {{/case}}
  {{#case "advanced"}}
Focus on optimization patterns.
  {{/case}}
{{/switch}}
```

### Conversation Templates

Create multi-turn conversation templates for complex interactions.

#### Conversation Template Format

```markdown
---
variables:
  topic: "{{topic}}"
  user_level: "{{user_level}}"
  language: "{{language}}"
---

## System
You are an expert {{language}} instructor. The student is {{user_level}} level.

Adapt your teaching style:
- **Beginner**: Use simple examples and explain basics
- **Intermediate**: Assume some knowledge, focus on patterns
- **Advanced**: Discuss edge cases and performance

## User
I want to learn {{topic}} in {{language}}. Can you explain the key concepts?

## Assistant  
I'd love to help you master {{topic}} in {{language}}! Since you're at {{user_level}} level, let me start with the core concepts.

{{#switch user_level}}
  {{#case "beginner"}}
Let's begin with the fundamentals...
  {{/case}}
  {{#case "intermediate"}}  
You probably know the basics, so let's dive into...
  {{/case}}
  {{#case "advanced"}}
Let's explore the advanced patterns and optimizations...
  {{/case}}
{{/switch}}

## User
{{user_question}}
```

#### Direct Template Loading in Messages

```rust
use responses::Messages;

// Load conversation template directly into Messages
let conversation = Messages::from_conversation_template(
    "prompts/conversations/learning_session.md",
    &serde_json::json!({
        "topic": "async programming",
        "language": "Rust",
        "user_level": "intermediate"
    })
)?;

// Use with client
let response = client
    .text()
    .model("gpt-4o")
    .messages(conversation)
    .send()
    .await?;
```

#### Using Conversation Templates

```rust
use responses::prompt::ConversationTemplate;

// Load and configure conversation template
let conversation = ConversationTemplate::load("prompts/conversations/learning_session.md")?
    .var("topic", "async programming")
    .var("language", "Rust")
    .var("user_level", "intermediate")
    .var("user_question", "How do I handle errors in async code?");

// Convert to Messages for API use
let messages = conversation.to_messages()?;

let response = client
    .text()
    .model("gpt-4o")
    .messages(messages)
    .send()
    .await?;
```

#### Messages Builder Integration

```rust
// Mix template and regular messages
let conversation = Messages::new()
    .system_from_md("prompts/coding_mentor.md")?
    .var("specialization", "systems programming")
    .with_locale("en", &["templates/locales"])?
    .user("I'm working on a performance-critical application")
    .assistant_from_md("prompts/responses/performance_intro.md")?
    .var("context", "systems programming")
    .user("What about memory allocation patterns?");
```

#### Advanced Conversation Features

```rust
// Conditional conversation flow
let debug_conversation = ConversationTemplate::load("prompts/conversations/debug_session.md")?
    .var("error_count", 3)
    .var("language", "Rust")
    .var("include_advanced_tips", true);

// Template inheritance for conversation families
let code_review_conversation = ConversationTemplate::load("prompts/conversations/code_review.md")?
    .var("review_type", "security")
    .var("language", "Rust")
    .var("complexity", "high");
```

## LLM-as-a-Judge

The LLM-as-a-Judge functionality enables automated evaluation of LLM outputs by using another LLM to assess whether responses match expected behavior. This is particularly useful for testing, quality assurance, and automated evaluation pipelines.

**Key Features:**
- ✅ **Zero Setup Required** - Default configuration works out of the box
- ✅ **Progressive Enhancement** - Start simple, add complexity when needed
- ✅ **Template Integration** - Full support for template-based evaluation criteria
- ✅ **i18n Support** - International evaluation through existing template system
- ✅ **Test Assertions** - Convenient macros for test integration
- ✅ **Flexible Prompts** - String prompts, template files, or template sets

### Basic Judge Usage

#### Simple Evaluation

```rust
use responses::{azure, Judge, Messages, Client};

let provider = azure().from_env()?.build()?;
let client = Client::new(provider);

// Works with OpenAI too
// let provider = openai().from_env()?.build()?;
// let client = Client::new(provider);

// Create judge and configure with custom evaluation prompt (users should tailor for their use-case)
let judge = Judge::new(client, "gpt-4o")
    .with_prompt(r#"You are a software developer reviewing code and responses. 
Your task is to evaluate whether an LLM's response matches the expected behavior from a developer's perspective.

Consider these factors:
1. Code accuracy and best practices
2. Implementation correctness  
3. Function calling and API usage
4. Technical completeness and quality
5. Appropriateness for development context

Always provide your evaluation as structured JSON with a boolean 'passes' field and detailed 'reasoning' string."#);

let conversation = Messages::new()
    .system("You are a helpful assistant")
    .user("What's 2+2?");

let response = client.text()
    .model("gpt-4o") 
    .messages(conversation.clone())
    .send().await?;

// Evaluate the response
let judgment = judge.evaluate(
    &conversation,
    &response,
    "Should provide correct mathematical answer"
).await?;

// Check the results
if judgment.passes {
    println!("✅ Response passed: {}", judgment.reasoning);
} else {
    println!("❌ Response failed: {}", judgment.reasoning);
}
```

#### Judge API

```rust
impl<P: Provider> Judge<P> {
    /// Create a new judge instance
    /// 
    /// Note: You must configure a prompt using one of the `with_*` methods before evaluation:
    /// - `.with_prompt(prompt)` for string prompts
    /// - `.with_template_file(path)` for template files  
    /// - `.with_template_directory(path, name)` for template sets
    pub fn new(client: Client<P>, model: impl Into<String>) -> Self;
    
    /// Use custom system prompt (plain string, no substitution)
    pub fn with_prompt(self, prompt: impl Into<String>) -> Self;
    
    /// Use template from file (supports variable substitution)
    pub fn with_template_file<T: AsRef<Path>>(self, path: T) -> Result<Self>;
    
    /// Use template from file with specific locale
    pub fn with_template_file_and_locale<T: AsRef<Path>>(
        self, 
        path: T, 
        locale: impl Into<String>
    ) -> Result<Self>;
    
    /// Use template directory with i18n support
    pub fn with_template_directory<T: AsRef<Path>>(
        self, 
        path: T, 
        template_name: impl Into<String>
    ) -> Result<Self>;
    
    /// Set locale for template directory usage
    pub fn with_locale(self, locale: impl Into<String>) -> Result<Self>;
    
    /// Set temperature for judgment consistency (default: 0.1)
    pub fn with_temperature(self, temp: f32) -> Self;
    
    /// Evaluate a conversation and response against expected behavior
    pub async fn evaluate(
        &self,
        conversation_history: &Messages,
        actual_response: &Response<String>, 
        expected_behavior: &str
    ) -> Result<Judgment>;
}
```

#### Custom Evaluation Prompts

```rust
// Custom string prompt
let strict_judge = Judge::new(client, "gpt-4o")
    .with_prompt(r#"You are a strict code reviewer. Evaluate responses for:
    - Code correctness and best practices
    - Security considerations
    - Performance implications
    - Documentation quality
    
    Provide structured JSON with 'passes' boolean and detailed 'reasoning'."#)
    .with_temperature(0.05); // Very consistent judgments

let judgment = strict_judge.evaluate(
    &conversation,
    &response, 
    "Should follow Rust best practices and be well-documented"
).await?;

// Fluent assertion methods
judgment.assert_passes().assert_confidence(0.8);
```

### Template-Based Evaluation

#### Template File Support

```markdown
<!-- prompts/judge/code_review.md -->
---
variables:
  language: "{{language}}"
  focus_area: "{{focus_area}}"
required_variables:
  - "language"
  - "focus_area"
---

# Code Review Evaluation

You are an expert {{language}} code reviewer specializing in {{focus_area}}.

## Evaluation Criteria
- Code correctness and functionality
- Adherence to {{language}} best practices
- {{focus_area}} specific requirements
- Code readability and maintainability

## Conversation History
{{conversation_history}}

## Actual Response
{{actual_response}}

## Expected Behavior
{{expected_behavior}}

## Task
Evaluate whether the actual response meets the expected behavior for {{focus_area}} in {{language}}.

Provide your judgment as structured JSON with:
- `passes`: boolean indicating if the response meets expectations
- `reasoning`: detailed explanation of your evaluation
- `confidence`: optional confidence score (0.0 to 1.0)
```

```rust
// Use template-based evaluation
let template_judge = Judge::new(client, "gpt-4o")
    .with_template_file("prompts/judge/code_review.md")?
    .with_temperature(0.2);

// Template variables are passed during evaluation
let judgment = template_judge.evaluate(
    &conversation,
    &response,
    "Should implement secure authentication with proper error handling"
).await?;

// The template automatically receives these variables:
// - conversation_history: formatted conversation
// - actual_response: formatted response with text and function calls  
// - expected_behavior: the expected behavior string
// - language: "Rust" (from your template vars)
// - focus_area: "security" (from your template vars)
```

#### Template Set with i18n

```rust
// Template directory structure:
// templates/
// ├── judge/
// │   ├── default.md
// │   └── security_review.md  
// └── locales/
//     ├── en/judge.yaml
//     ├── es/judge.yaml
//     └── ja/judge.yaml

let multilingual_judge = Judge::new(client, "gpt-4o")
    .with_template_directory("templates", "judge/security_review")?
    .with_locale("es")?  // Spanish evaluation
    .with_temperature(0.1);

let judgment = multilingual_judge.evaluate(
    &conversation,
    &response,
    "Debe implementar autenticación segura con manejo adecuado de errores"
).await?;
```

### Test Assertions

#### Assertion Macros

```rust
use responses::{assert_passes, assert_fails, assert_confidence};

// Assert judgment passes
assert_passes!(judgment);

// Assert judgment fails
assert_fails!(judgment);

// Assert minimum confidence level
assert_confidence!(judgment, 0.8);

// Custom assertion messages
assert_passes!(judgment, "Code review should pass for this implementation");
```

#### Fluent Assertion Methods

```rust
// Chain assertion methods
let judgment = judge.evaluate(&conversation, &response, "Should be helpful").await?;

judgment
    .assert_passes()           // Assert it passes
    .assert_confidence(0.7);   // Assert minimum confidence

// Or check individual conditions
if judgment.passes {
    println!("✅ Evaluation passed: {}", judgment.reasoning);
    if let Some(confidence) = judgment.confidence {
        println!("📊 Confidence: {:.1}%", confidence * 100.0);
    }
} else {
    println!("❌ Evaluation failed: {}", judgment.reasoning);
}
```

#### Test Integration Example

```rust
#[tokio::test]
async fn test_helpful_response_evaluation() -> responses::Result<()> {
    let provider = azure().from_env()?.build()?;
    let client = Client::new(provider);
    let judge = Judge::new(client.clone(), "gpt-4o")
        .with_prompt("You are an expert evaluator. Judge whether responses are helpful and correct.");
    
    let conversation = Messages::new()
        .system("You are a helpful assistant")
        .user("How do I reverse a string in Rust?");
    
    let response = client.text()
        .model("gpt-4o")
        .messages(conversation.clone())
        .send().await?;
    
    // Evaluate response quality
    let judgment = judge.evaluate(
        &conversation,
        &response,
        "Should provide correct, helpful code example with explanation"
    ).await?;
    
    // Use assertion macros
    assert_passes!(judgment);
    assert_confidence!(judgment, 0.7);
    
    Ok(())
}
```

#### Progressive Usage Patterns

**Level 1: Simple String Prompts**
```rust
let judge = Judge::new(client, "gpt-4o")
    .with_prompt("You are a helpful evaluator. Judge whether responses are correct and helpful.");
let judgment = judge.evaluate(&conversation, &response, "Should be correct").await?;
assert_passes!(judgment);
```

**Level 2: Custom Prompts**
```rust
let judge = Judge::new(client, "gpt-4o")
    .with_prompt("You are a strict evaluator...")
    .with_temperature(0.05);
    
let judgment = judge.evaluate(&conversation, &response, "Must follow best practices").await?;
judgment.assert_passes().assert_confidence(0.8);
```

**Level 3: Template-Based with i18n**
```rust
let judge = Judge::new(client, "gpt-4o")
    .with_template_directory("templates", "judge/code_review")?
    .with_locale("es")?
    .with_temperature(0.1);
    
let judgment = judge.evaluate(&conversation, &response, "Debe seguir mejores prácticas").await?;
assert_passes!(judgment);
```

## Temperature Control

Temperature support has been added across all request builders for controlling response randomness and creativity.

### Temperature in Text and Structured Requests

```rust
// Low temperature for deterministic outputs
let response = client.text()
    .model("gpt-4o")
    .temperature(0.1)  // Focused and consistent
    .system("Provide a technical explanation")
    .user("Explain how HashMap works in Rust")
    .send().await?;

// High temperature for creative outputs  
let story = client.structured::<CreativeStory>()
    .model("gpt-4o")
    .temperature(0.8)  // More varied and creative
    .system("You are a creative storyteller")
    .user("Write an original short story")
    .send().await?;

// Balanced temperature for general use
let balanced = client.text()
    .model("gpt-4o")
    .temperature(0.5)  // Balanced creativity and consistency
    .user("Explain the pros and cons of microservices")
    .send().await?;
```

### Temperature Guidelines

- **0.0 - 0.2**: Highly deterministic, focused responses. Ideal for:
  - Code generation and technical explanations
  - Mathematical computations
  - Factual Q&A
  - LLM-as-a-judge evaluations (default: 0.1)

- **0.3 - 0.7**: Balanced creativity and consistency. Good for:
  - General conversation
  - Educational content
  - Problem-solving discussions
  - Business writing

- **0.8 - 1.0**: Highly creative and varied responses. Best for:
  - Creative writing and storytelling
  - Brainstorming sessions
  - Artistic content generation
  - Exploring alternative perspectives

### Judge Temperature

```rust
// Default: consistent evaluations
let consistent_judge = Judge::new(client, "gpt-4o")  // Uses 0.1 temperature by default
    .with_prompt("You are an expert evaluator...");

// Custom temperature for specific evaluation needs
let flexible_judge = Judge::new(client, "gpt-4o")
    .with_prompt("Evaluate creative writing quality...")
    .with_temperature(0.3);  // Slightly more flexible judgments

// Very strict evaluation
let strict_judge = Judge::new(client, "gpt-4o")
    .with_prompt("Strict code review criteria...")
    .with_temperature(0.05);  // Maximum consistency
```

## Error Handling

Template rendering uses strict error handling, returning errors for missing variables or i18n keys instead of placeholder text.

```rust
// Missing variables return errors
let template = PromptTemplate::from_content("Hello {{name}}")?;
let result = template.render(&json!({}));
// Result: Err(Error::TemplateVariableNotFound { name: "name" })

// Missing i18n keys return errors
let template = PromptTemplate::from_content("{{i18n \"missing.key\"}}")?
    .with_locale("en", &["locales"])?;
let result = template.render(&json!({}));
// Result: Err(Error::I18nKeyNotFound { key: "missing.key", locale: "en" })

// Handle template errors appropriately
match template.render(&vars) {
    Ok(rendered) => println!("Template: {}", rendered),
    Err(Error::TemplateVariableNotFound { name }) => {
        eprintln!("Missing variable: {}", name);
        // Provide the missing variable and retry
    }
    Err(Error::I18nKeyNotFound { key, locale, template_file, available_keys }) => {
        eprintln!("Missing i18n key '{}' in locale '{}'", key, locale);
        if let Some(file) = template_file {
            eprintln!("  Template: {}", file);
        }
        if let Some(keys) = available_keys {
            eprintln!("  Available keys: {}", keys.join(", "));
        }
        // Add the key to your locale files
    }
    Err(e) => eprintln!("Template error: {}", e),
}
```

### Error Types

```rust
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
```

### Response Handling

The `Response<T>` structure provides direct access to response data and function calls. Both text responses and function calls are equally valid response types - function-only responses are normal behavior, not error conditions.

```rust
#[derive(Clone, Debug)]
pub struct Response<T> {
    /// The text response from the model (can be `None` for function-only responses)
    pub message: Option<Result<T, Refusal>>,
    
    /// Function calls requested by the model
    pub function_calls: Vec<OutputFunctionCall>,
}

impl<T> Response<T> {
    /// Returns true if the response contains a text message
    pub fn has_text_message(&self) -> bool;
    
    /// Returns true if the response contains function calls
    pub fn has_function_calls(&self) -> bool;
    
    /// Returns true if this is a function-only response (normal behavior)
    pub fn is_function_only(&self) -> bool;
    
    /// Returns true if response contains both text and function calls
    pub fn has_both_text_and_functions(&self) -> bool;
    
    /// Get a reference to the successful text message, if it exists
    pub fn text_message(&self) -> Option<&T>;
    
    /// Get a reference to the refusal, if the message was refused
    pub fn refusal(&self) -> Option<&Refusal>;
    
    /// Get the number of function calls in this response
    pub fn function_call_count(&self) -> usize;
}
```

#### Basic Response Handling

```rust
let response = client
    .text()
    .model("gpt-4o")
    .user("Tell me a joke")
    .send()
    .await?;

// Using new helper methods for clearer response handling
if let Some(text) = response.text_message() {
    println!("✅ Text response: {}", text);
} else if let Some(refusal) = response.refusal() {
    println!("❌ Model refused: {}", refusal);
} else if response.is_function_only() {
    println!("🔧 Function-only response (normal behavior)");
} else {
    println!("ℹ️ Empty response");
}

// Check for function calls
if response.has_function_calls() {
    println!("🚀 Model requested {} function calls", response.function_call_count());
    for call in &response.function_calls {
        println!("Function: {} with args: {}", call.name, call.arguments);
    }
}

// Handle mixed responses (both text and functions)
if response.has_both_text_and_functions() {
    println!("📝 Response contains both text and function calls");
}
```

### Refusal Type

```rust
#[derive(Clone, Debug)]
pub struct Refusal(String);

impl Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Implementation details
    }
}
```


## Examples

### Complete Text Generation Example

```rust
use responses::{azure, Messages, Client};

#[tokio::main]
async fn main() -> responses::Result<()> {
    let provider = azure().from_env()?.build()?;
    let client = Client::new(provider);
    
    // Works with OpenAI too
    // let provider = openai().from_env()?.build()?;
    // let client = Client::new(provider);
    
    let conversation = Messages::new()
        .system("You are a helpful programming mentor")
        .user("I'm learning Rust. What are the key concepts I should focus on?")
        .assistant("Great choice! Key Rust concepts include ownership, borrowing, lifetimes, and pattern matching.")
        .user("Can you elaborate on ownership?");
    
    let response = client
        .text()
        .model("gpt-4o")
        .messages(conversation)
        .send()
        .await?;
    
    if let Some(Ok(text)) = response.message {
        println!("Mentor says: {}", text);
    }
    
    Ok(())
}
```

### Complete Structured Output Example

```rust
use responses::{azure, Messages, Client};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Clone, Debug, JsonSchema, Deserialize)]
struct CodeAnalysis {
    complexity: String,
    performance: String,
    suggestions: Vec<String>,
    rating: i32,
}

#[tokio::main]
async fn main() -> responses::Result<()> {
    let provider = azure().from_env()?.build()?;
    let client = Client::new(provider);
    
    // Works with OpenAI too
    // let provider = openai().from_env()?.build()?;
    // let client = Client::new(provider);
    
    let analysis = client
        .structured::<CodeAnalysis>()
        .model("gpt-4o")
        .system("You are a code analysis expert. Provide structured analysis.")
        .user(r#"Analyze this Python function:

def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
"#)
        .send()
        .await?;
    
    if let Some(Ok(analysis_data)) = analysis.message {
        println!("Complexity: {}", analysis_data.complexity);
        println!("Performance: {}", analysis_data.performance);
        println!("Rating: {}/10", analysis_data.rating);
        for suggestion in analysis_data.suggestions {
            println!("- {}", suggestion);
        }
    }
    
    Ok(())
}
```

### Tool Usage Example

```rust
use responses::{azure, tool, types::ToolChoice, Client};
use responses::provider::ProviderBuilder;

#[tool]
/// Get weather information for a city
async fn get_weather(
    city: String,
    country: Option<String>, // Optional parameter
) -> responses::Result<String> {
    // Simulate weather API call
    let location = match country {
        Some(c) => format!("{}, {}", city, c),
        None => city,
    };
    Ok(format!("Weather in {}: Sunny, 22°C", location))
}

#[tool]
/// Perform mathematical calculations
async fn calculate(
    a: f64,
    b: f64,
    operation: String, // add, subtract, multiply, divide
) -> responses::Result<f64> {
    // Implementation here
    match operation.as_str() {
        "add" => Ok(a + b),
        "subtract" => Ok(a - b), 
        "multiply" => Ok(a * b),
        "divide" => {
            if b != 0.0 {
                Ok(a / b)
            } else {
                Err(responses::error::Error::InvalidResponse("Cannot divide by zero".to_string()))
            }
        }
        _ => Err(responses::error::Error::InvalidResponse(format!("Unknown operation: {}", operation))),
    }
}

#[tokio::main]
async fn main() -> responses::Result<()> {
    let provider = azure().from_env()?.build()?;
    let client = Client::new(provider);
    
    let response = client
        .text()
        .model("gpt-4o")
        .system("You are a helpful assistant with weather and calculation capabilities")
        .user("What's the weather in Tokyo, Japan and what's 15 * 23?")
        .tools(vec![get_weather_handler().tool(), calculate_handler().tool()])
        .tool_choice(ToolChoice::Auto)
        .send()
        .await?;
    
    // Handle function calls - no manual name matching needed!
    let weather_handler = get_weather_handler();
    let calc_handler = calculate_handler();
    
    for function_call in response.function_calls {
        // Each handler automatically checks if it can handle the call
        if let Some(result) = weather_handler.invoke(&function_call).await? {
            println!("🌤️ Weather result: {}", result);
        } else if let Some(result) = calc_handler.invoke(&function_call).await? {
            println!("🧮 Calculation result: {}", result);
        } else {
            println!("❓ No handler found for: {}", function_call.name);
        }
    }
    
    Ok(())
}
```

### Enhanced Function Calling Example

```rust
use responses::{azure, tool, types::ToolChoice, Client};

#[tool]
/// Get current weather information
async fn get_weather(
    city: String,
    country: Option<String>,
    units: Option<String>,
) -> Result<WeatherResponse> {
    // Simulate weather API call
    Ok(WeatherResponse {
        temperature: 22.0,
        condition: "Sunny".to_string(),
        location: city,
    })
}

#[tool]
/// Perform mathematical calculations
async fn calculate(
    expression: String,
    operation: String,
) -> Result<f64> {
    // Simulate calculation
    match operation.as_str() {
        "multiply" => Ok(345.0),
        _ => Ok(0.0)
    }
}

#[tokio::main]
async fn main() -> responses::Result<()> {
    let provider = azure().from_env()?.build()?;
    let client = Client::new(provider);
    
    let response = client
        .text()
        .model("gpt-4o")
        .system("You are a versatile assistant with weather and calculation capabilities")
        .user("What's the weather in Tokyo and what's 15 * 23?")
        .tools(vec![get_weather_handler(), calculate_handler()])
        .tool_choice(ToolChoice::Auto)
        .send()
        .await?;
    
    // Handle assistant message
    if let Some(Ok(message)) = response.message {
        println!("Assistant: {}", message);
    }
    
    // Handle function calls - no manual name matching needed!
    let weather_handler = get_weather_handler();
    let calc_handler = calculate_handler();
    
    for function_call in response.function_calls {
        // Each handler automatically checks if it can handle the call
        if let Some(result) = weather_handler.invoke(&function_call).await? {
            println!("🌤️ Weather result: {:?}", result);
        } else if let Some(result) = calc_handler.invoke(&function_call).await? {
            println!("🧮 Calculation result: {}", result);
        } else {
            println!("❓ No handler found for: {}", function_call.name);
        }
    }
    
    Ok(())
}
```

### Complete LLM-as-a-Judge Example

This example demonstrates automated evaluation of code generation responses using LLM-as-a-Judge.

```rust
use responses::{azure, Judge, Messages, Client, assert_passes, assert_confidence};
use serde::Deserialize;
use responses::schemars::JsonSchema;

#[derive(Clone, Debug, JsonSchema, Deserialize)]
struct CodeAnalysis {
    language: String,
    functionality: String,
    best_practices_score: i32,
    suggestions: Vec<String>,
}

#[tokio::main]
async fn main() -> responses::Result<()> {
    let provider = azure().from_env()?.build()?;
    let client = Client::new(provider);
    
    // === SETUP CONVERSATION ===
    let conversation = Messages::new()
        .system(r#"You are an expert Rust programmer. Provide clean, efficient, 
        and well-documented code following Rust best practices."#)
        .user("Write a function to implement a binary search algorithm in Rust. Include error handling and documentation.");
    
    // === GENERATE RESPONSE ===
    let response = client.structured::<CodeAnalysis>()
        .model("gpt-4o")
        .temperature(0.3)  // Balanced creativity for code generation
        .messages(conversation.clone())
        .send().await?;
    
    println!("Generated response: {:?}", response);
    
    // === SETUP JUDGE FOR EVALUATION ===
    
    // 1. Basic evaluation with simple prompt
    let basic_judge = Judge::new(client.clone(), "gpt-4o")
        .with_prompt("You are an expert evaluator. Judge whether responses meet expectations.");
    
    let basic_judgment = basic_judge.evaluate(
        &conversation,
        &response,
        r#"Should provide:
        1. Correct binary search implementation
        2. Proper error handling
        3. Clear documentation
        4. Rust best practices (ownership, borrowing, etc.)
        5. Comprehensive example usage"#
    ).await?;
    
    println!("📊 Basic Evaluation:");
    println!("  Passes: {}", basic_judgment.passes);
    println!("  Reasoning: {}", basic_judgment.reasoning);
    if let Some(confidence) = basic_judgment.confidence {
        println!("  Confidence: {:.1}%", confidence * 100.0);
    }
    
    // 2. Detailed code review with custom prompt
    let code_review_judge = Judge::new(client.clone(), "gpt-4o")
        .with_prompt(r#"You are a senior Rust developer conducting a thorough code review.
        
        Evaluate the response for:
        1. **Correctness**: Algorithm implementation is correct and handles edge cases
        2. **Safety**: Proper use of Rust's ownership system and memory safety
        3. **Performance**: Efficient implementation with good time/space complexity
        4. **Style**: Follows Rust naming conventions and idiomatic patterns
        5. **Documentation**: Clear comments and documentation
        6. **Testing**: Includes or suggests appropriate test cases
        
        Be strict in your evaluation. Code must meet production standards.
        Provide structured JSON with 'passes' boolean and detailed 'reasoning'."#)
        .with_temperature(0.05); // Very consistent for code review
    
    let detailed_judgment = code_review_judge.evaluate(
        &conversation,
        &response,
        "Must be production-ready Rust code following all best practices"
    ).await?;
    
    println!("\n🔍 Detailed Code Review:");
    println!("  Passes: {}", detailed_judgment.passes);
    println!("  Reasoning: {}", detailed_judgment.reasoning);
    
    // 3. Performance-focused evaluation
    let performance_judge = Judge::new(client.clone(), "gpt-4o")
        .with_prompt(r#"You are a performance optimization expert.
        
        Focus specifically on:
        - Time complexity analysis
        - Space complexity considerations
        - Memory allocation patterns
        - Potential performance bottlenecks
        - Optimization opportunities
        
        Provide structured JSON evaluation."#)
        .with_temperature(0.1);
    
    let performance_judgment = performance_judge.evaluate(
        &conversation,
        &response,
        "Should demonstrate optimal O(log n) binary search with minimal memory overhead"
    ).await?;
    
    println!("\n⚡ Performance Analysis:");
    println!("  Passes: {}", performance_judgment.passes);
    println!("  Reasoning: {}", performance_judgment.reasoning);
    
    // === TEST ASSERTIONS ===
    
    println!("\n🧪 Running Test Assertions...");
    
    // Basic assertions
    assert_passes!(basic_judgment, "Basic functionality check failed");
    assert_confidence!(basic_judgment, 0.6);
    
    // Fluent assertion chains
    detailed_judgment
        .assert_passes()
        .assert_confidence(0.7);
    
    // Performance validation
    if performance_judgment.passes {
        println!("✅ Performance evaluation passed");
        assert_confidence!(performance_judgment, 0.5);
    } else {
        println!("⚠️ Performance concerns identified: {}", performance_judgment.reasoning);
    }
    
    // === AGGREGATE RESULTS ===
    
    let evaluations = vec![
        ("Basic Functionality", &basic_judgment),
        ("Code Review", &detailed_judgment),
        ("Performance", &performance_judgment),
    ];
    
    let passed_count = evaluations.iter().filter(|(_, j)| j.passes).count();
    let total_count = evaluations.len();
    
    println!("\n📋 Evaluation Summary:");
    println!("  Overall: {}/{} evaluations passed", passed_count, total_count);
    
    let avg_confidence: f64 = evaluations.iter()
        .filter_map(|(_, j)| j.confidence)
        .sum::<f64>() / evaluations.len() as f64;
    
    println!("  Average Confidence: {:.1}%", avg_confidence * 100.0);
    
    if passed_count == total_count {
        println!("🎉 All evaluations passed! Code is ready for production.");
    } else {
        println!("🔧 Some evaluations failed. Review the feedback above.");
    }
    
    // === TEMPERATURE COMPARISON ===
    
    println!("\n🌡️ Testing Temperature Effects on Evaluation:");
    
    let temperatures = vec![0.0, 0.1, 0.3, 0.7];
    
    for temp in temperatures {
        let temp_judge = Judge::new(client.clone(), "gpt-4o")
            .with_prompt("You are an expert evaluator. Judge overall code quality.")
            .with_temperature(temp);
        
        let temp_judgment = temp_judge.evaluate(
            &conversation,
            &response,
            "Evaluate overall code quality"
        ).await?;
        
        println!("  Temperature {:.1}: {} ({})", 
                temp, 
                if temp_judgment.passes { "PASS" } else { "FAIL" },
                if let Some(conf) = temp_judgment.confidence { 
                    format!("{:.1}%", conf * 100.0) 
                } else { 
                    "N/A".to_string() 
                }
        );
    }
    
    Ok(())
}
```

This comprehensive example demonstrates:

- **🎯 Multiple Evaluation Perspectives**: Basic functionality, detailed code review, and performance analysis
- **🔧 Custom Judge Configuration**: Different prompts and temperatures for different evaluation criteria  
- **✅ Test Integration**: Using assertion macros and fluent methods for automated testing
- **📊 Evaluation Analytics**: Aggregating results and calculating metrics
- **🌡️ Temperature Analysis**: Comparing evaluation consistency across different temperature settings
- **🔄 Production Workflow**: Complete pipeline from generation to evaluation to decision making

### Complete Enterprise Template System Example

This example showcases all the advanced template features working together in a production environment.

#### Template Directory Structure

```
templates/
├── system_prompts/
│   └── ai_assistant.md          # Main system prompt
├── shared/
│   ├── header.md               # Shared header
│   ├── footer.md               # Shared footer
│   └── guidelines.md           # Common guidelines
├── conversations/
│   └── onboarding.md           # Multi-turn conversation
└── locales/
    ├── en/
    │   └── system.yaml         # English strings
    └── es/
        └── system.yaml         # Spanish strings
```

#### Main System Template (`templates/system_prompts/ai_assistant.md`)

```markdown
---
required_variables:
  - "user_name"
  - "user_role"
  - "specialization"
includes:
  - "shared/header.md"
  - "shared/guidelines.md"
  - "shared/footer.md"
i18n_key: "system.main_prompt"
variables:
  assistant_name: "Claude"
  experience_level: "expert"
---

{{> shared/header.md}}

# {{i18n "system.title"}}

{{i18n "system.greeting" user_name=user_name assistant_name=assistant_name}}

## {{i18n "system.role_assignment"}}
You are an {{experience_level}} {{specialization}} assistant working with {{user_name}}, who is a {{user_role}}.

{{> shared/guidelines.md}}

## {{i18n "system.capabilities"}}
{{i18n "system.capabilities_text" specialization=specialization}}

{{> shared/footer.md}}
```

#### Locale Files

```yaml
# locales/en/system.yaml
system:
  title: "AI Assistant Configuration"
  greeting: "Hello {user_name}, I'm {assistant_name}, your AI assistant."
  role_assignment: "Role Assignment"
  capabilities: "My Capabilities"
  capabilities_text: "I specialize in {specialization} and can help with complex problem-solving, analysis, and guidance."

shared:
  header: "Welcome to your personalized AI assistant experience."
  footer: "I'm here to help you succeed. Let's get started!"
  
guidelines:
  title: "Interaction Guidelines"
  be_helpful: "Always be helpful and supportive"
  ask_questions: "Ask clarifying questions when needed"
  provide_examples: "Provide concrete examples when possible"
```

```yaml
# locales/es/system.yaml
system:
  title: "Configuración del Asistente IA"
  greeting: "Hola {user_name}, soy {assistant_name}, tu asistente de IA."
  role_assignment: "Asignación de Rol"
  capabilities: "Mis Capacidades"
  capabilities_text: "Me especializo en {specialization} y puedo ayudar con resolución de problemas complejos, análisis y orientación."

shared:
  header: "Bienvenido a tu experiencia personalizada de asistente IA."
  footer: "Estoy aquí para ayudarte a tener éxito. ¡Comencemos!"
  
guidelines:
  title: "Pautas de Interacción"
  be_helpful: "Siempre ser útil y solidario"
  ask_questions: "Hacer preguntas aclaratorias cuando sea necesario"
  provide_examples: "Proporcionar ejemplos concretos cuando sea posible"
```

#### Production Application Code

```rust
use responses::{azure, Client, Messages};
use responses::prompt::TemplateSet;
use serde_json::json;

#[tokio::main]
async fn main() -> responses::Result<()> {
    // === INITIALIZE TEMPLATE SYSTEM ===
    
    let template_set = TemplateSet::from_dir("templates")?;
    
    // Validate all templates at startup
    println!("📋 Available templates: {:?}", template_set.list_templates());
    println!("💬 Available conversations: {:?}", template_set.list_conversations());
    
    // === MULTI-LOCALE SUPPORT ===
    // Note: Manual environment variable reading (application code)
    
    let user_locale = std::env::var("USER_LOCALE").unwrap_or_else(|_| "en".to_string());
    let localized_templates = template_set.with_locale(&user_locale)?;
    
    println!("🌐 Using locale: {}", localized_templates.current_locale());
    
    // === CLIENT SETUP ===
    
    let provider = azure().from_env()?.build()?;
    let client = Client::new(provider);
    
    // === RENDER SYSTEM PROMPT WITH VALIDATION ===
    
    let user_vars = json!({
        "user_name": "Dr. Sarah Chen",
        "user_role": "Senior Data Scientist",
        "specialization": "machine learning"
    });
    
    // This automatically validates required variables and includes
    let system_prompt = localized_templates.render("system_prompts/ai_assistant", &user_vars)?;
    
    println!("✅ System prompt rendered successfully");
    println!("📄 Length: {} characters", system_prompt.len());
    
    // === FLUENT API WITH TEMPLATES ===
    
    let conversation = Messages::new()
        .system(system_prompt)
        .user("I'm working on a recommendation system project. What approach would you suggest for handling cold start problems?");
    
    // === CONVERSATION TEMPLATE INTEGRATION ===
    
    let onboarding_conversation = localized_templates.render_conversation("onboarding", &user_vars)?;
    
    let extended_conversation = conversation
        .extend(onboarding_conversation)
        .user("Let's start with collaborative filtering approaches.");
    
    // === API REQUEST ===
    
    let response = client
        .text()
        .model("gpt-4o")
        .messages(extended_conversation)
        .send()
        .await?;
    
    if let Some(Ok(text)) = response.message {
        println!("🤖 Assistant Response:");
        println!("{}", text);
    }
    
    Ok(())
}

// === ERROR HANDLING WITH TEMPLATE VALIDATION ===

async fn handle_template_errors() -> responses::Result<()> {
    let template_set = TemplateSet::from_dir("templates")?;
    
    // This will show specific validation errors
    let incomplete_vars = json!({
        "user_name": "Alice"
        // Missing required: user_role, specialization
    });
    
    match template_set.render("system_prompts/ai_assistant", &incomplete_vars) {
        Ok(_) => println!("✅ Template rendered successfully"),
        Err(responses::Error::RequiredVariablesMissing { variables }) => {
            println!("❌ Missing required variables: {:?}", variables);
            println!("💡 Please provide: user_role, specialization");
        }
        Err(e) => println!("❌ Template error: {}", e),
    }
    
    Ok(())
}

// === PRODUCTION DEPLOYMENT VALIDATION ===

async fn validate_production_templates() -> responses::Result<()> {
    println!("🔍 Validating production template environment...");
    
    // Load templates
    let template_set = TemplateSet::from_dir("templates")?;
    
    // Test each locale
    for locale in ["en", "es", "ja"] {
        println!("  🌐 Testing locale: {}", locale);
        
        let localized_set = template_set.with_locale(locale)?;
        
        // Test sample rendering
        let test_vars = json!({
            "user_name": "Test User",
            "user_role": "Developer", 
            "specialization": "software engineering"
        });
        
        let _rendered = localized_set.render("system_prompts/ai_assistant", &test_vars)?;
        println!("    ✅ Locale {} validated", locale);
    }
    
    println!("🎉 All templates validated for production deployment!");
    Ok(())
}
```

This comprehensive example demonstrates:

- **🔒 Production Validation**: Required variables and includes validation
- **🌐 Multi-locale Support**: Automatic locale switching for all templates  
- **📁 Organized Architecture**: Clean template directory structure with shared components
- **🔗 Dependency Management**: Automatic include resolution with base path
- **⚡ High Performance**: Template caching and efficient rendering
- **🛡️ Error Handling**: Comprehensive validation with helpful error messages
- **🔄 Fluent Integration**: Seamless integration with Messages API and client requests

### Complete OpenAI Provider Example

This example demonstrates using the OpenAI provider with all the same features as Azure OpenAI.

```rust
use responses::{openai, Client, Messages};
use responses::schemars::JsonSchema;
use serde::Deserialize;

#[derive(Clone, Debug, JsonSchema, Deserialize)]
struct TaskAnalysis {
    priority: String,
    estimated_hours: f64,
    dependencies: Vec<String>,
    risk_level: String,
}

#[tokio::main]
async fn main() -> responses::Result<()> {
    // === OPENAI PROVIDER SETUP ===
    
    // From environment variable (OPENAI_API_KEY)
    let provider = openai()
        .from_env()?
        .build()?;
    let client = Client::new(provider);
    
    // Or with explicit API key
    // let provider = openai()
    //     .api_key("sk-your-api-key-here")
    //     .build()?;
    // let client = Client::new(provider);
    
    // === TEXT GENERATION ===
    
    let conversation = Messages::new()
        .system("You are a project management assistant")
        .user("I need to implement user authentication in my web app");
    
    let text_response = client
        .text()
        .model("gpt-4")  // OpenAI model names
        .messages(conversation.clone())
        .temperature(0.7)
        .send()
        .await?;
    
    if let Some(Ok(text)) = text_response.message {
        println!("🤖 OpenAI Response: {}", text);
    }
    
    // === STRUCTURED OUTPUT ===
    
    let structured_response = client
        .structured::<TaskAnalysis>()
        .model("gpt-4")
        .system("Analyze project tasks and provide structured breakdown")
        .user("Analyze the task: 'Implement user authentication with JWT tokens, password reset, and email verification'")
        .temperature(0.3)
        .send()
        .await?;
    
    if let Some(Ok(analysis)) = structured_response.message {
        println!("📊 Task Analysis:");
        println!("  Priority: {}", analysis.priority);
        println!("  Estimated Hours: {}", analysis.estimated_hours);
        println!("  Risk Level: {}", analysis.risk_level);
        println!("  Dependencies: {:?}", analysis.dependencies);
    }
    
    // === TEMPLATE INTEGRATION ===
    
    // Templates work identically with OpenAI
    let template_response = client
        .text()
        .model("gpt-4")
        .system_from_md("prompts/project_manager.md")?
        .var("project_type", "web application")
        .var("tech_stack", "React + Node.js")
        .var("team_size", 3)
        .user("What's the best approach for implementing real-time features?")
        .send()
        .await?;
    
    // === FUNCTION CALLING ===
    
    use responses::{tool, types::ToolChoice};
    
    #[tool]
    /// Estimate project timeline based on complexity
    async fn estimate_timeline(
        task_description: String,
        complexity: String, // low, medium, high
        team_size: i32,
    ) -> responses::Result<String> {
        let base_hours = match complexity.as_str() {
            "low" => 20,
            "medium" => 50,
            "high" => 120,
            _ => 40,
        };
        
        let adjusted_hours = base_hours / team_size.max(1);
        Ok(format!("Estimated timeline: {} hours for '{}' (complexity: {})", 
                  adjusted_hours, task_description, complexity))
    }
    
    let function_response = client
        .text()
        .model("gpt-4")
        .system("You help estimate project timelines. Use the estimation tool when asked about timelines.")
        .user("How long would it take to implement a user dashboard with data visualization?")
        .tools(vec![estimate_timeline_handler()])
        .tool_choice(ToolChoice::Auto)
        .send()
        .await?;
    
    // Handle function calls
    let timeline_handler = estimate_timeline_handler();
    for function_call in function_response.function_calls {
        if let Some(result) = timeline_handler.invoke(&function_call).await? {
            println!("⏱️ Timeline Estimate: {}", result);
        }
    }
    
    // === LLM-AS-A-JUDGE WITH OPENAI ===
    
    use responses::Judge;
    
    let judge = Judge::new(client.clone(), "gpt-4")
        .with_prompt("You are an expert software architect. Evaluate whether technical solutions are well-designed and follow best practices.")
        .with_temperature(0.1);
    
    let evaluation_conversation = Messages::new()
        .system("You are a senior developer")
        .user("Should I use microservices for a small e-commerce site with 3 developers?");
    
    let eval_response = client
        .text()
        .model("gpt-4")
        .messages(evaluation_conversation.clone())
        .send()
        .await?;
    
    let judgment = judge.evaluate(
        &evaluation_conversation,
        &eval_response,
        "Should provide appropriate architectural guidance considering team size and project scale"
    ).await?;
    
    println!("🧑‍⚖️ Evaluation Result: {}", if judgment.passes { "PASS" } else { "FAIL" });
    println!("📝 Reasoning: {}", judgment.reasoning);
    
    Ok(())
}
```

**Key differences when using OpenAI vs Azure OpenAI:**

- **Endpoint**: Fixed to `https://api.openai.com/v1/responses`
- **Authentication**: Uses `Bearer` token instead of `api-key` header
- **Configuration**: Only requires API key (no resource/version parameters)
- **Model names**: Use OpenAI model names (`gpt-4`, `gpt-4-turbo`, etc.)
- **Same API**: All other functionality (templates, structured outputs, tools, judge) works identically

**Environment Setup:**
```bash
# Set your OpenAI API key
export OPENAI_API_KEY="sk-your-api-key-here"

# All other environment variables work the same
export RESPONSES_LOCALES_PATH="templates/locales"
```

---

This documentation covers the complete public API surface of the responses library, including:

- **🌐 Dual Provider Support**: Both Azure OpenAI and plain OpenAI with identical APIs
- **🎯 LLM-as-a-Judge**: Automated evaluation of LLM outputs with template support and i18n
- **🌡️ Temperature Control**: Fine-grained control over response randomness across all builders  
- **🔧 Enhanced Function Calling**: Streamlined function definition and execution with `#[tool]` macro
- **📝 Flexible Template System**: Markdown templates with variables, i18n, and composition
- **💬 Fluent Conversation Management**: Builder patterns for complex multi-turn interactions

The library provides a unified interface for both providers, so you can switch between Azure OpenAI and OpenAI with minimal code changes - just swap the provider configuration while keeping all other functionality identical.

For more examples, check the `examples/` directory in the repository.
