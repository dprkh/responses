use dotenv::dotenv;
use responses::{azure, Client};
use responses::provider::ProviderBuilder;
// Use the re-exported schemars to avoid version conflicts
use responses::schemars::JsonSchema;
use serde::Deserialize;

#[derive(Clone, Debug, JsonSchema, Deserialize)]
struct Step {
    explanation: String,
    output: String,
}

#[derive(Clone, Debug, JsonSchema, Deserialize)]
struct MathResponse {
    steps: Vec<Step>,
    final_answer: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();

    let provider = azure().from_env().unwrap().build().unwrap();
    let client = Client::new(provider);

    let response = client
        .structured::<MathResponse>()
        .model("gpt-4o")
        .user("Solve 2 + 3 * 2")
        .send()
        .await
        .unwrap();

    println!("{response:?}");
}
