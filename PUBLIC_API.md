# Responses API Documentation

A comprehensive Rust library for Azure OpenAI's Responses API, providing a modern fluent DSL with template support for text generation and structured outputs.

## Table of Contents

- [Getting Started](#getting-started)
- [Client Setup](#client-setup)
  - [Azure Configuration](#azure-configuration)
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
- [Error Handling](#error-handling)
- [Examples](#examples)

## Getting Started

Add this to your `Cargo.toml`:

```toml
[dependencies]
responses = "0.1"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["rt", "macros"] }
```

Set up environment variables for Azure OpenAI:

```bash
export AZURE_OPENAI_API_KEY="your-api-key"
export AZURE_OPENAI_RESOURCE="your-resource-name"
export AZURE_OPENAI_API_VERSION="2025-03-01-preview"
```

## Client Setup

### Azure Configuration

This library provides Azure OpenAI integration only.

#### Quick Setup

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

#### Manual Configuration

```rust
use responses::{azure, providers::AzureConfig, Client};
use responses::provider::ProviderBuilder;

let config = AzureConfig {
    api_key: "your-api-key".to_string(),
    resource: "your-resource".to_string(),
    api_version: "2025-03-01-preview".to_string(),
};

let provider = azure()
    .with_config(config)
    .build()?;
let client = Client::new(provider);
```

#### Builder Pattern

```rust
use responses::{azure, Client};
use responses::provider::ProviderBuilder;

let provider = azure()
    .api_key("your-api-key")
    .resource("your-resource")
    .api_version("2025-03-01-preview")
    .build()?;
let client = Client::new(provider);
```

## Core APIs

### Text Generation

The `TextRequestBuilder` provides a fluent API for generating text responses.

#### Basic Text Generation

```rust
use responses::{azure, Client};
use responses::provider::ProviderBuilder;

let provider = azure().from_env()?.build()?;
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

#### `TextRequestBuilder` API

```rust
impl<P: Provider> TextRequestBuilder<P> {
    pub fn model<S: Into<String>>(self, model: S) -> Self;
    pub fn system<S: Into<String>>(self, content: S) -> Self;
    pub fn user<S: Into<String>>(self, content: S) -> Self;
    pub fn assistant<S: Into<String>>(self, content: S) -> Self;
    pub fn developer<S: Into<String>>(self, content: S) -> Self;
    
    pub fn tools(self, tools: Vec<Tool>) -> Self;
    pub fn tool_choice(self, choice: ToolChoice) -> Self;
    pub fn safety_identifier<S: Into<String>>(self, id: S) -> Self;
    
    pub fn messages(self, messages: Messages) -> Self;
    pub fn continue_conversation(self, messages: &Messages) -> Self;
    pub fn from_messages(self, messages: Vec<InputMessage>) -> Self;
    pub fn add_messages<I, S>(self, messages: I) -> Self 
    where
        I: IntoIterator<Item = (Role, S)>,
        S: Into<String>;
    
    pub async fn send(self) -> Result<Response<String>>;
}
```

#### Complex Conversation

```rust
let response = client
    .text()
    .model("gpt-4o")
    .system("You are a helpful coding assistant")
    .user("How do I implement a binary search?")
    .assistant("Here's how to implement binary search in Rust...")
    .user("Can you show me the recursive version?")
    .send()
    .await?;
```

### Structured Outputs

The `StructuredRequestBuilder` provides type-safe structured outputs using JSON Schema.

#### Defining Response Types

```rust
use serde::{Deserialize};
use schemars::JsonSchema;

#[derive(Clone, Debug, JsonSchema, Deserialize)]
struct WeatherResponse {
    temperature: f64,
    condition: String,
    humidity: i32,
    location: String,
}
```

#### Basic Structured Request

```rust
let weather = client
    .structured::<WeatherResponse>()
    .model("gpt-4o")
    .system("Provide accurate weather information")
    .user("What's the weather like in San Francisco?")
    .send()
    .await?;

if let Some(Ok(weather_data)) = weather.message {
    println!("Temperature: {}°C", weather_data.temperature);
    println!("Condition: {}", weather_data.condition);
}
```

#### Custom Schema Names

```rust
let response = client
    .structured_with_name::<WeatherResponse>("DetailedWeather".to_string())
    .model("gpt-4o")
    .user("Get weather for New York")
    .send()
    .await?;
```

#### `StructuredRequestBuilder` API

```rust
impl<P: Provider, T> StructuredRequestBuilder<P, T> 
where T: JsonSchema + for<'a> Deserialize<'a>
{
    pub fn model<S: Into<String>>(self, model: S) -> Self;
    pub fn system<S: Into<String>>(self, content: S) -> Self;
    pub fn user<S: Into<String>>(self, content: S) -> Self;
    pub fn assistant<S: Into<String>>(self, content: S) -> Self;
    pub fn developer<S: Into<String>>(self, content: S) -> Self;
    
    pub fn tools(self, tools: Vec<Tool>) -> Self;
    pub fn tool_choice(self, choice: ToolChoice) -> Self;
    pub fn safety_identifier<S: Into<String>>(self, id: S) -> Self;
    
    pub fn messages(self, messages: Messages) -> Self;
    pub fn continue_conversation(self, messages: &Messages) -> Self;
    pub fn from_messages(self, messages: Vec<InputMessage>) -> Self;
    pub fn add_messages<I, S>(self, messages: I) -> Self 
    where
        I: IntoIterator<Item = (Role, S)>,
        S: Into<String>;
    
    pub async fn send(self) -> Result<Response<T>>;
}
```

### Enhanced Function Calling

The library provides an advanced function calling system that automatically deserializes function parameters, eliminating the manual JSON parsing required in basic function calls.

**Key Features:**
- ✅ **Built-in schema generation** - No need to manually define JSON schemas
- ✅ **Automatic parameter parsing** from JSON
- ✅ **Type-safe function execution** with `.invoke()`
- ✅ **Azure OpenAI compatible** - Generates correct `parameters` field structure
- ✅ **Required vs optional fields** - Automatically handles `Option<T>` types
- ✅ **Enhanced type support** - Arrays (`Vec<T>`), maps (`HashMap`, `BTreeMap`), all integer types
- ✅ **Improved error context** - Function-specific error messages with parameter details
- ✅ **No manual name matching** - Handlers return `Option<T>`

#### Quick Function Call Example

```rust
use responses::{azure, tool, types::ToolChoice};

#[tool]
/// Get weather information for a city
async fn get_weather(
    city: String,
    country: Option<String>,
    units: Option<String>,
) -> Result<WeatherResponse> {
    // Your weather API implementation here
    Ok(WeatherResponse {
        temperature: 22.0,
        condition: "Sunny".to_string(),
        location: city,
    })
}

let provider = azure().from_env()?.build()?;
let client = Client::new(provider);

let response = client
    .text()
    .model("gpt-4o")
    .system("You are a helpful weather assistant")
    .user("What's the weather in Paris?")
    .tools(vec![get_weather_handler().into()])  // Cleaner with From trait
    .tool_choice(ToolChoice::Auto)
    .send()
    .await?;

// Handle function calls
for function_call in response.function_calls {
    println!("Called: {}", function_call.name);
    // Process function call as needed
}
```




## Conversation Management

### Messages Builder

The `Messages` type provides comprehensive conversation management with builder patterns.

#### Basic Messages Builder

```rust
use responses::{Messages, messages};

let conversation = Messages::new()
    .system("You are a helpful assistant")
    .user("Hello, how are you?")
    .assistant("I'm doing well, thank you!")
    .user("What can you help me with?");

println!("Conversation has {} messages", conversation.len());
```

#### Using the Macro

```rust
let conversation = messages! {
    system: "You are a coding expert",
    user: "Explain recursion",
    assistant: "Recursion is when a function calls itself...",
    user: "Show me an example"
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
    
    // Accessors
    pub fn into_inputs(self) -> Vec<Input>;
    pub fn inputs(&self) -> &[Input];
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

#### Saving and Reusing Conversations

```rust
// Build a conversation template
let template = Messages::new()
    .system("You are a helpful tutor")
    .user("I need help with math");

// Use the template multiple times
let algebra_conversation = template.clone()
    .user("Explain linear equations");

let geometry_conversation = template.clone()
    .user("Explain the Pythagorean theorem");

// Send requests
let algebra_response = client
    .text()
    .model("gpt-4o")
    .messages(algebra_conversation)
    .send()
    .await?;
```

#### Continuing Conversations

```rust
// Start a conversation
let initial_conversation = Messages::new()
    .system("You are a programming mentor")
    .user("I'm learning Rust");

let first_response = client
    .text()
    .model("gpt-4o")
    .messages(initial_conversation.clone())
    .send()
    .await?;

// Continue without consuming the original
let continued_response = client
    .text()
    .model("gpt-4o")
    .continue_conversation(&initial_conversation)
    .user("What about memory management?")
    .send()
    .await?;
```

### Advanced Message Operations

#### Conversation Slicing

```rust
let full_conversation = Messages::new()
    .system("System prompt")
    .user("Question 1")
    .assistant("Answer 1")
    .user("Question 2")
    .assistant("Answer 2")
    .user("Question 3");

// Get the last 2 messages
let recent_context = full_conversation.take_last(2);

// Get only user messages
let user_questions = full_conversation.filter_by_role(Role::User);

// Get first 3 messages (system + first exchange)
let initial_context = full_conversation.take_first(3);
```

#### Bulk Message Operations

```rust
let bulk_messages = vec![
    (Role::System, "You are a helpful assistant"),
    (Role::User, "Hello"),
    (Role::Assistant, "Hi there!"),
    (Role::User, "How are you?"),
];

let conversation = Messages::new()
    .add_messages(bulk_messages);
```


## Function Calling with #[tool]

The library provides the most ergonomic function calling experience through the `#[tool]` attribute macro. Simply annotate any function, and the library automatically handles everything else.

**What you get:**
- ✅ **Built-in schema generation** - Automatically creates Azure OpenAI compatible schemas
- ✅ **Smart type mapping** - `String` → `"string"`, `Option<T>` → optional fields
- ✅ **Automatic parameter parsing** from JSON
- ✅ **Type-safe function execution** with `.invoke()`
- ✅ **No manual name matching required** - handlers return `Option<T>`
- ✅ **Concurrent execution support** with `tokio::spawn`
- ✅ **Clone-able handlers** for sharing across tasks
- ✅ **Context parameter support**
- ✅ **Custom tool naming**
- ✅ **Documentation from docstrings**

### Basic Function Definition

```rust
use responses::tool;

#[tool]
/// Get current weather information for a city
async fn get_weather(
    city: String,
    country: Option<String>,
    context: &AppContext,
) -> Result<WeatherResponse> {
    // Your implementation here
    Ok(WeatherResponse {
        temperature: 22.0,
        condition: "Sunny".to_string(),
        location: city,
    })
}

#[tool("weather_lookup")]  // Custom tool name
/// Get weather data with custom name
async fn get_weather_custom(city: String) -> Result<String> {
    Ok(format!("Weather for {}: Sunny, 22°C", city))
}
```

### Simple Function Execution

Each handler automatically checks if it can handle a function call and returns `Option<T>`:

```rust
use responses::{tool, types::ToolChoice};

#[tool]
/// Get weather information for a city
async fn get_weather(
    city: String,
    country: Option<String>,
    context: &AppContext,
) -> Result<WeatherResponse> {
    // Your weather API implementation
    Ok(WeatherResponse {
        temperature: 22.0,
        condition: "Sunny".to_string(),
        location: city,
    })
}

#[tool("weather_lookup")]
/// Custom weather lookup
async fn get_weather_custom(city: String) -> Result<String> {
    Ok(format!("Weather for {}: Sunny, 22°C", city))
}

// Use in API request
let response = client
    .text()
    .model("gpt-4o")
    .system("You are a weather assistant")
    .user("What's the weather in Paris and Tokyo?")
    .tools(vec![get_weather_handler().into(), get_weather_custom_handler().into()])
    .tool_choice(ToolChoice::Auto)
    .send()
    .await?;

// Handle function calls - each handler automatically checks function names
let weather_handler = get_weather_handler();
let custom_handler = get_weather_custom_handler();

for call in response.function_calls {
    // Try each handler - they return Some(result) if they match, None if not
    if let Some(result) = weather_handler.invoke(&call, &context).await? {
        println!("✅ Weather result: {:?}", result);
    } else if let Some(result) = custom_handler.invoke(&call).await? {
        println!("✅ Custom result: {}", result);
    } else {
        println!("❓ No handler for: {}", call.name);
    }
}
```

### Concurrent Execution

For optimal performance with multiple function calls, spawn handlers concurrently:

```rust
// Helper function to handle a single call
async fn handle_function_call(
    call: OutputFunctionCall,
    weather_handler: GetWeatherHandler,
    custom_handler: GetWeatherCustomHandler,
    context: AppContext,
) -> Result<()> {
    // Try each handler - they automatically check if they can handle the call
    if let Some(result) = weather_handler.invoke(&call, &context).await? {
        println!("✅ Weather: {:?}", result);
    } else if let Some(result) = custom_handler.invoke(&call).await? {
        println!("✅ Custom: {}", result);
    } else {
        println!("❓ No handler for: {}", call.name);
    }
    Ok(())
}

// Single call - execute directly
if response.function_calls.len() == 1 {
    let call = &response.function_calls[0];
    if let Some(result) = weather_handler.invoke(call, &context).await? {
        println!("✅ Weather: {:?}", result);
    } else if let Some(result) = custom_handler.invoke(call).await? {
        println!("✅ Custom: {}", result);
    } else {
        println!("❓ No handler for: {}", call.name);
    }
}
// Multiple calls - spawn concurrently  
else if response.function_calls.len() > 1 {
    println!("🚀 Executing {} function calls concurrently...", response.function_calls.len());
    
    let mut handles = Vec::new();
    for call in response.function_calls {
        let weather_h = weather_handler.clone();
        let custom_h = custom_handler.clone();
        let ctx = context.clone();
        
        let handle = tokio::spawn(async move {
            handle_function_call(call, weather_h, custom_h, ctx).await
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    for handle in handles {
        if let Err(e) = handle.await? {
            eprintln!("❌ Error: {}", e);
        }
    }
    
    println!("⚡ All function calls completed!");
}
```

### Key Features

- **No Manual Matching**: Handlers automatically validate function names and return `Option<T>`
- **Type-safe**: Automatic parameter parsing and validation from JSON
- **Simple**: Uses function name as tool name by default
- **Customizable**: Override tool name with `#[tool("custom_name")]`
- **Documentation**: Uses Rust docstrings for tool descriptions
- **Context support**: Detects and handles context parameters
- **Concurrent**: Built-in support for parallel execution with `tokio::spawn`
- **Clone-able**: Generated handlers can be shared across tasks
- **Clean APIs**: Each handler's `invoke` method returns `Option<T>` if it matches
- **From Trait**: Automatic conversion from handlers to tools with `.into()`

### Cleaner Tool Usage

The `#[tool]` macro now generates `From<Handler> for Tool` implementations, making tool usage cleaner:

```rust
// Before (explicit)
.tools(vec![handler.tool()])

// After (cleaner with From trait)  
.tools(vec![handler.into()])

// Both approaches work, use whichever you prefer
.tools(vec![
    weather_handler.into(),          // Using From trait
    calculate_handler.tool(),        // Explicit method
])
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
    .system_prompt(r#"You are a senior software architect.

Guidelines:
- Consider scalability and performance
- Suggest appropriate technologies
- Ask clarifying questions when needed
- Provide concrete examples"#)
    .user_message(r#"I need to design a chat system that handles:
- 10,000 concurrent users
- Message history storage
- File sharing
- Group chats up to 100 members

What architecture would you recommend?"#)
    .assistant_response(r#"For this scale, I'd recommend:

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
    pub fn with_locale(mut self, locale: &str) -> Result<Self>;
    
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
    .with_locale("es")?
    .var("role", "asistente")
    .var("domain", "programación");
```

### Template Set Management

The `TemplateSet` provides enterprise-grade template management with organized directory structures, base path resolution, and locale switching.

#### TemplateSet API

```rust
impl TemplateSet {
    // === CREATION AND LOADING ===
    
    /// Load all templates from a directory
    pub fn from_dir<P: AsRef<Path>>(dir: P) -> Result<Self>;
    
    // === LOCALE MANAGEMENT ===
    
    /// Switch locale for all templates in the set
    pub fn with_locale(mut self, locale: &str) -> Result<Self>;
    
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

// === LOADING TEMPLATE SETS ===

let template_set = TemplateSet::from_dir("templates")?;

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

// Switch to Spanish locale for all templates
let spanish_set = template_set.with_locale("es")?;
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

Templates support full internationalization with locale-specific strings, pluralization, and formatting.

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
{{i18n "current_tasks" count=task_count tasks=(plural task_count "tasks")}}

Progress: {{format_number progress style="percent"}}

{{#if_locale "ar"}}
<div dir="rtl">المحتوى بالعربية</div>
{{/if_locale}}
```

#### Using Localized Templates

```rust
// Auto-detect locale from environment (LANG=es)
let response = client
    .text()
    .model("gpt-4o")
    .system_from_md("prompts/assistant.md")?
    .var("role", "asistente")
    .var("task_count", 3)
    .send()
    .await?;

// Explicit locale
let template = PromptTemplate::load("prompts/system.md")?
    .with_locale("ja")?
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
    .with_locale("en")?
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

## Error Handling

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
    
    #[error("i18n key not found: {key} in locale {locale}\nHelp: Add the key '{key}' to your locale file at locales/{locale}/*.yaml")]
    I18nKeyNotFound { key: String, locale: String },
    
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

```rust
let response = client
    .text()
    .model("gpt-4o")
    .user("Tell me a joke")
    .send()
    .await?;

match response.message {
    Some(Ok(text)) => println!("Success: {}", text),
    Some(Err(refusal)) => println!("Model refused: {}", refusal),
    None => println!("No response generated"),
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

---

This documentation covers the complete public API surface of the responses library, including the new enhanced function calling capabilities. For more examples, check the `examples/` directory in the repository.
