use dotenv::dotenv;
use responses::azure;
use schemars::JsonSchema;
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

    let client = azure().from_env().unwrap().build_client().unwrap();

    let response = client
        .structured::<MathResponse>()
        .model("gpt-4o")
        .user("Solve 2 + 3 * 2")
        .send()
        .await
        .unwrap();

    println!("{response:?}");
}
