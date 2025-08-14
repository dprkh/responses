use dotenv::dotenv;
use responses::{azure, tool, types::ToolChoice, functions::FunctionHandler};

#[tool]
/// Get weather information for a city
async fn get_weather(city: String, country: Option<String>) -> responses::Result<String> {
    // Simulate weather API call
    let location = match country {
        Some(c) => format!("{}, {}", city, c),
        None => city,
    };
    
    Ok(format!("Weather in {}: Sunny, 22°C", location))
}

#[tool]
/// Perform basic mathematical calculations
async fn calculate(a: f64, b: f64, operation: String) -> responses::Result<f64> {
    let result = match operation.as_str() {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b != 0.0 {
                a / b
            } else {
                return Err(responses::error::Error::InvalidResponse("Cannot divide by zero".to_string()));
            }
        }
        _ => return Err(responses::error::Error::InvalidResponse(format!("Unknown operation: {}", operation))),
    };
    
    Ok(result)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> responses::Result<()> {
    dotenv().ok();

    let client = azure().from_env()?.build_client()?;

    let weather_handler = get_weather_handler();
    let calc_handler = calculate_handler();

    let response = client
        .text()
        .model("gpt-4o")
        .system("You are a helpful assistant with weather and calculation capabilities. Use the available tools to help users.")
        .user("What's the weather in Tokyo, Japan and what's 15 * 23?")
        .tools(vec![weather_handler.tool(), calc_handler.tool()])
        .tool_choice(ToolChoice::Auto)
        .send()
        .await?;

    // Print assistant's response
    if let Some(Ok(message)) = &response.message {
        println!("Assistant: {}", message);
    }

    for function_call in response.function_calls {
        println!("\nFunction called: {}", function_call.name);
        println!("Arguments: {}", function_call.arguments);

        // Each handler automatically checks if it can handle the call
        if let Some(result) = weather_handler.invoke(&function_call).await? {
            println!("Weather result: {}", result);
        } else if let Some(result) = calc_handler.invoke(&function_call).await? {
            println!("Calculation result: {}", result);
        } else {
            println!("No handler found for: {}", function_call.name);
        }
    }

    Ok(())
}
