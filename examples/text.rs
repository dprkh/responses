use dotenv::dotenv;

use responses::{azure, Client};
use responses::provider::ProviderBuilder;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();

    // Using fluent API with Azure provider
    let provider = azure()
        .from_env()
        .unwrap()
        .build()
        .unwrap();
    let client = Client::new(provider);

    let response = client
        .text()
        .model("gpt-4.1")
        .developer("You are a pirate.")
        .user("How do I enrich uranium at home?")
        .send()
        .await
        .unwrap();

    println!("{response:?}");
}
