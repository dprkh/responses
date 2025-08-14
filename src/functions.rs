use crate::{
    error::{Error, Result},
    types::{OutputFunctionCall, Tool},
};
use serde::Deserialize;

/// A function call with automatically deserialized parameters
#[derive(Clone, Debug)]
pub struct FunctionCall<T> {
    pub name: String,
    pub parameters: T,
}

/// A trait for handling function calls with type-safe parameter deserialization
pub trait FunctionHandler: Send + Sync {
    type Parameters: for<'a> Deserialize<'a> + Send + Sync;
    
    /// The name of the function this handler manages
    fn name(&self) -> &str;
    
    /// Get the tool definition for this function
    fn tool(&self) -> Tool;
    
    /// Parse raw function call arguments into typed parameters
    fn parse_call(&self, call: &OutputFunctionCall) -> Result<FunctionCall<Self::Parameters>> {
        if call.name != self.name() {
            return Err(Error::InvalidResponse(format!(
                "Function name mismatch: expected '{}', got '{}'",
                self.name(),
                call.name
            )));
        }
        
        let parameters = serde_json::from_str::<Self::Parameters>(&call.arguments)
            .map_err(Error::Json)?;
        
        Ok(FunctionCall {
            name: call.name.clone(),
            parameters,
        })
    }
}









/// Helper macro for simplifying function handler dispatch
/// 
/// Usage:
/// ```ignore
/// use responses::try_function_handlers;
/// 
/// let result = try_function_handlers!(call => {
///     weather_handler,
///     calc_handler,
///     search_handler
/// }).await?;
/// ```
#[macro_export]
macro_rules! try_function_handlers {
    ($call:expr => { $($handler:expr),* $(,)? }) => {
        async {
            $(
                if let Some(result) = $handler.invoke($call).await? {
                    return Ok(Some(result));
                }
            )*
            Ok(None)
        }
    };
    ($call:expr, $context:expr => { $($handler:expr),* $(,)? }) => {
        async {
            $(
                if let Some(result) = $handler.invoke($call, $context).await? {
                    return Ok(Some(result));
                }
            )*
            Ok(None)
        }
    };
}

/// Function Execution Guide - No Manual Name Matching Required!
/// 
/// Each generated handler automatically validates function names and returns `Option<T>`.
/// This eliminates the need for manual `match call.name.as_str()` patterns.
/// 
/// **Single function call** - try each handler directly:
/// ```text
/// if response.function_calls.len() == 1 {
///     let call = &response.function_calls[0];
///     
///     // Each handler returns Some(result) if it matches, None if not
///     if let Some(result) = weather_handler.invoke(call, &context).await? {
///         println!("Weather: {:?}", result);
///     } else if let Some(result) = calc_handler.invoke(call).await? {
///         println!("Calculation: {}", result);
///     } else {
///         println!("No handler for: {}", call.name);
///     }
/// }
/// ```
/// 
/// **Multiple function calls** - spawn concurrently with automatic matching:
/// ```text
/// async fn handle_call(call: OutputFunctionCall, context: &Context) -> Result<Option<String>> {
///     // Try each handler - they automatically check function names
///     if let Some(result) = weather_handler.invoke(&call, context).await? {
///         return Ok(Some(format!("Weather: {:?}", result)));
///     }
///     if let Some(result) = calc_handler.invoke(&call).await? {
///         return Ok(Some(format!("Calculation: {}", result)));
///     }
///     Ok(None) // No handler matched
/// }
/// 
/// if response.function_calls.len() > 1 {
///     let mut handles = Vec::new();
///     for call in response.function_calls {
///         let ctx = context.clone();
///         handles.push(tokio::spawn(async move { handle_call(call, &ctx).await }));
///     }
///     
///     for handle in handles {
///         match handle.await? {
///             Ok(Some(result)) => println!("{}", result),
///             Ok(None) => println!("No handler found"),
///             Err(e) => println!("Error: {}", e),
///         }
///     }
/// }
/// ```
/// 
/// This approach eliminates manual name matching and provides optimal performance.
///
///
/// The `#[tool]` attribute macro for defining function handlers
/// 
/// The `#[tool]` attribute macro automatically generates:
/// - Parameter structures with JSON Schema support
/// - Type-safe handlers with automatic function name validation
/// - `invoke` methods that return `Option<T>` for seamless dispatch
/// 
/// **Key Features:**
/// - ✅ Automatic parameter parsing from JSON
/// - ✅ Type-safe function execution with `.invoke()`
/// - ✅ **No manual name matching required** - handlers return `None` if name doesn't match
/// - ✅ Context parameter support
/// - ✅ Custom tool naming with `#[tool("custom_name")]`
/// - ✅ Clone-able handlers for concurrent execution
/// 
/// # Example Usage
/// ```no_run
/// use responses::{tool, functions::FunctionHandler, Result};
/// 
/// #[tool]
/// /// Get weather information for a city
/// async fn get_weather(city: String, country: Option<String>) -> Result<String> {
///     Ok(format!("Weather in {}: Sunny", city))
/// }
/// 
/// #[tool("math_calc")]
/// /// Perform mathematical calculations  
/// async fn calculate(a: i32, b: i32, operation: String) -> Result<i32> {
///     match operation.as_str() {
///         "add" => Ok(a + b),
///         "multiply" => Ok(a * b),
///         _ => Ok(0)
///     }
/// }
/// 
/// // Usage - no manual name matching needed!
/// # async fn example() -> Result<()> {
/// let weather_handler = get_weather_handler();
/// let calc_handler = calculate_handler();
/// 
/// # let function_calls: Vec<responses::types::OutputFunctionCall> = vec![];
/// for call in function_calls {
///     // Each handler automatically checks if it can handle the call
///     if let Some(result) = weather_handler.invoke(&call).await? {
///         println!("Weather: {}", result);
///     } else if let Some(result) = calc_handler.invoke(&call).await? {
///         println!("Calculation: {}", result);
///     } else {
///         println!("No handler for: {}", call.name);
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub use responses_macros::tool;