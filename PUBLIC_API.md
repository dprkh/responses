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
- [Function Handling](#function-handling)
  - [Function Handlers](#function-handlers)
  - [Function Request Builder](#function-request-builder)
  - [Function Call Processing](#function-call-processing)
  - [Function Macros](#function-macros)
- [Multiline Message Support](#multiline-message-support)
- [Prompt Templates](#prompt-templates)
  - [Markdown Template System](#markdown-template-system)
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
export AZURE_OPENAI_API_VERSION="2024-02-15-preview"
```

## Client Setup

### Azure Configuration

This library provides Azure OpenAI integration only.

#### Quick Setup

```rust
use responses::azure;

#[tokio::main]
async fn main() -> responses::Result<()> {
    let client = azure()
        .from_env()?
        .build_client()?;
    
    // Use client for requests...
    Ok(())
}
```

#### Manual Configuration

```rust
use responses::{azure, providers::AzureConfig};

let config = AzureConfig {
    api_key: "your-api-key".to_string(),
    resource: "your-resource".to_string(),
    api_version: "2024-02-15-preview".to_string(),
};

let client = azure()
    .with_config(config)
    .build_client()?;
```

#### Builder Pattern

```rust
let client = azure()
    .api_key("your-api-key")
    .resource("your-resource")
    .api_version("2024-02-15-preview")
    .build()?
    .build_client()?;
```

## Core APIs

### Text Generation

The `TextRequestBuilder` provides a fluent API for generating text responses.

#### Basic Text Generation

```rust
use responses::azure;

let client = azure().from_env()?.build_client()?;

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

let client = azure().from_env()?.build_client()?;

let response = client
    .text()
    .model("gpt-4o")
    .system("You are a helpful weather assistant")
    .user("What's the weather in Paris?")
    .tools(vec![get_weather_handler()])
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
    
    // Multiline message helpers
    pub fn system_prompt<S: Into<String>>(self, content: S) -> Self;
    pub fn user_message<S: Into<String>>(self, content: S) -> Self;
    pub fn assistant_response<S: Into<String>>(self, content: S) -> Self;
    pub fn developer_note<S: Into<String>>(self, content: S) -> Self;
    
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
    .tools(vec![get_weather_handler().tool(), get_weather_custom_handler().tool()])
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

Store your prompts in `.md` files with YAML frontmatter for configuration and variables.

#### Basic Template Structure

```markdown
---
variables:
  role: "{{role}}"
  domain: "{{domain}}"
  experience_years: "{{experience_years}}"
locale_key: "coding_assistant"
---

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
```

#### Loading Templates in Code

```rust
use responses::{azure, Messages};
use responses::prompt::template::MarkdownPrompt;

let client = azure().from_env()?.build_client()?;

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

#### Template API

```rust
// Load template from file
let template = MarkdownPrompt::load("prompts/system.md")?;

// Set variables
let template = template
    .var("role", "assistant")
    .var("expertise", "machine learning")
    .var("active", true);

// Apply locale
let template = template.with_locale("es")?;

// Render to string
let rendered = template.render()?;
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
let template = MarkdownPrompt::load("prompts/system.md")?
    .with_locale("ja")?
    .var("role", "アシスタント");
```

### Template Variables and Composition

Templates support variables, conditionals, loops, and includes for powerful composition.

#### Variable Substitution

```rust
// Simple variables
let template = MarkdownPrompt::load("prompts/greeting.md")?
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

let template = MarkdownPrompt::load("prompts/profile.md")?
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
use responses::prompt::conversation::ConversationTemplate;

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
use responses::{azure, Messages};

#[tokio::main]
async fn main() -> responses::Result<()> {
    let client = azure().from_env()?.build_client()?;
    
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
use responses::{azure, Messages};
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
    let client = azure().from_env()?.build_client()?;
    
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
use responses::{azure, tool, types::ToolChoice};

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
    let client = azure().from_env()?.build_client()?;
    
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
use responses::{azure, tool, types::ToolChoice};

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
    let client = azure().from_env()?.build_client()?;
    
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

---

This documentation covers the complete public API surface of the responses library, including the new enhanced function calling capabilities. For more examples, check the `examples/` directory in the repository.