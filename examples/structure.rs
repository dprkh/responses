use dotenv::dotenv;

use responses::{
    Azure, Options,
    types::{Input, InputMessage, Role},
};

use schemars::JsonSchema;

use serde::Deserialize;

#[allow(unused)]
#[derive(Clone, Debug, JsonSchema, Deserialize)]
struct Step {
    explanation: String,

    output: String,
}

#[allow(unused)]
#[derive(Clone, Debug, JsonSchema, Deserialize)]
struct MathResponse {
    steps: Vec<Step>,

    final_answer: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();

    let azure = Azure::from_env();

    let response = azure
        //
        .structure::<MathResponse>(
            //
            "MathResponse".to_owned(),
            //
            Options {
                model: Some("gpt-4.1".to_owned()),

                input: Some(vec![Input::Message(InputMessage {
                    role: Role::User,
                    content: "Solve 2 + 3 * 2".to_owned(),
                })]),

                ..Default::default()
            },
        )
        //
        .await
        //
        .unwrap();

    println!("{response:?}");
}
