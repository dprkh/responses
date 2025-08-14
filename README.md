# responses

A comprehensive Rust library for Azure OpenAI's Responses API with fluent DSL, conversation management, function calling, and i18n template support.

## Features

- **Azure OpenAI Integration**: Built exclusively for Azure OpenAI services
- **Fluent DSL**: Modern, chainable API for building requests
- **Text & Structured Outputs**: Support for both text generation and JSON Schema-based structured outputs
- **Conversation Management**: Build, save, and reuse complex conversation histories
- **Enhanced Function Calling**: Declarative function definitions with type-safe handlers
- **Template System**: Markdown templates with YAML frontmatter and i18n support
- **Error Handling**: Comprehensive error types with detailed context

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
responses = "0.1"
schemars = "1.0"  # For structured outputs
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["rt", "macros"] }
```

Set up environment variables:

```bash
export AZURE_OPENAI_API_KEY="your-api-key"
export AZURE_OPENAI_RESOURCE="your-resource-name"
export AZURE_OPENAI_API_VERSION="2024-02-15-preview"
```

## Basic Usage

### Text Generation

```rust
use responses::azure;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = azure()
        .from_env()?
        .build_client()?;

    let response = client
        .text()
        .model("gpt-4o")
        .system("You are a helpful assistant")
        .user("Explain Rust ownership")
        .send()
        .await?;

    println!("{}", response.content);
    Ok(())
}
```

### Structured Outputs

```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(JsonSchema, Deserialize, Debug)]
struct CodeReview {
    rating: i32,
    issues: Vec<String>,
    suggestions: Vec<String>,
}

let review = client
    .structure::<CodeReview>("code_review")
    .model("gpt-4o")
    .system("You are a code reviewer")
    .user("Review this Rust code: fn main() { println!(\"Hello\"); }")
    .send()
    .await?;

println!("Rating: {}/10", review.content.rating);
```

## Conversation Management

Build and reuse complex conversation histories:

```rust
use responses::Messages;

// Build conversation
let conversation = Messages::new()
    .system("You are a Rust tutor")
    .user("What's ownership?")
    .assistant("Ownership is Rust's memory management system...")
    .user("Can you give an example?");

// Save for reuse
let saved_conversation = conversation.clone();

// Use in request
let response = client
    .text()
    .model("gpt-4o")
    .messages(conversation)
    .send()
    .await?;

// Continue later
let continued = client
    .text()
    .continue_conversation(&saved_conversation)
    .user("What about borrowing?")
    .send()
    .await?;
```

### Conversation Features

- **Fluent Builder**: `.system()`, `.user()`, `.assistant()`, `.developer()`
- **Persistence**: Clone and reuse conversations
- **Continuation**: `.continue_conversation()` without consuming original
- **Operations**: `.take_last(n)`, `.filter_by_role()`, `.extend()`
- **Macro Support**: `messages! { system: "...", user: "..." }`

## Enhanced Function Calling

Define functions declaratively with type-safe handlers:

```rust
use responses::tool;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(JsonSchema, Deserialize)]
struct WeatherQuery {
    location: String,
    unit: Option<String>,
}

#[tool]
async fn get_weather(params: WeatherQuery) -> String {
    format!("Weather in {}: 72°F, sunny", params.location)
}

let handler = get_weather_handler();

let response = client
    .text()
    .model("gpt-4o")
    .system("You are a weather assistant")
    .user("What's the weather in San Francisco?")
    .tools(vec![handler.tool()])
    .send()
    .await?;

// Handle function calls manually
for call in response.function_calls {
    if let Some(result) = handler.invoke(&call).await? {
        println!("Weather result: {}", result);
    }
}
```

## Template System

### Basic Templates

```rust
use responses::prompt::template::PromptTemplate;
use serde_json::json;

let template = PromptTemplate::from_content(r#"---
variables:
  role: "assistant"
required_variables:
  - "domain"
---

You are a helpful {{role}} specializing in {{domain}}."#)?;

let vars = json!({ "domain": "machine learning" });
let rendered = template.render(&vars)?;
```

### Conversation Templates

```rust
use responses::prompt::template::ConversationTemplate;

let conversation_template = r#"---
variables:
  topic: "{{topic}}"
  level: "{{level}}"
---

## System
You are teaching {{topic}} to {{level}} students.

## User  
How do I get started with {{topic}}?

## Assistant
Great question! Let me explain {{topic}} basics for {{level}} learners."#;

let conversation = ConversationTemplate::load_from_content(conversation_template)?;
let messages = conversation.render(&json!({
    "topic": "async Rust",
    "level": "intermediate"
}))?;
```

### Template Features

- **YAML Frontmatter**: Variables, required fields, i18n keys
- **Variable Substitution**: `{{variable}}` and `{{object.nested}}`
- **Role Mapping**: `## System`, `## User`, `## Assistant`, `## Developer`
- **i18n Support**: Hierarchical locale fallback (es-MX → es → en)
- **Application Scale**: `TemplateSet` for directory-based template management
- **Integration**: Template methods on Messages and request builders

### Application-Scale Templates

```rust
use responses::prompt::template::TemplateSet;

// Load all templates from directory
let templates = TemplateSet::from_dir("templates")?
    .with_locale("es")?;

// Render individual templates
let content = templates.render("system_prompt", &vars)?;

// Render conversation templates
let messages = templates.render_conversation("onboarding", &vars)?;
```

## Error Handling

Comprehensive error types with context:

```rust
match client.text().send().await {
    Ok(response) => println!("{}", response.content),
    Err(responses::Error::PromptFileRead { path, source }) => {
        eprintln!("Failed to read template {}: {}", path, source);
    }
    Err(responses::Error::RequiredVariablesMissing { variables }) => {
        eprintln!("Missing required variables: {:?}", variables);
    }
    Err(e) => eprintln!("Request failed: {}", e),
}
```

## Configuration

### Environment Variables

```bash
export AZURE_OPENAI_API_KEY="your-api-key"
export AZURE_OPENAI_RESOURCE="your-resource-name"  
export AZURE_OPENAI_API_VERSION="2024-02-15-preview"
```

### Manual Configuration

```rust
use responses::{azure, AzureConfig};

let config = AzureConfig {
    api_key: "key".to_string(),
    resource: "resource".to_string(),
    api_version: "2024-02-15-preview".to_string(),
};

let client = azure()
    .with_config(config)
    .build_client()?;
```

### Builder Pattern

```rust
let client = azure()
    .api_key("your-key")
    .resource("your-resource")
    .api_version("2024-02-15-preview")
    .build()?
    .build_client()?;
```

## Examples

The `examples/` directory contains comprehensive examples:

- **Basic Usage**: Text generation and structured outputs
- **Conversation Management**: Building and reusing conversations
- **Function Calling**: Declarative tool definitions
- **Template System**: Markdown templates with i18n
- **Full Workflow**: Complete template → render → send pipeline

Run examples with:

```bash
cargo run --example conversation_management
cargo run --example template_showcase
cargo run --example full_template_workflow
```

## Documentation

See [PUBLIC_API.md](PUBLIC_API.md) for complete API documentation including:

- Detailed API reference
- Advanced usage patterns
- Template system guide
- Function calling patterns
- Error handling strategies
- i18n configuration

## License

MIT