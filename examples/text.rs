use dotenv::dotenv;

use responses::{
    Azure, Options,
    types::{Input, InputMessage, Role},
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();

    let azure = Azure::from_env();

    let response = azure
        //
        .text(Options {
            model: Some("gpt-4.1".to_owned()),

            input: Some(vec![
                Input::Message(InputMessage {
                    role: Role::Developer,
                    content: "You are a pirate.".to_owned(),
                }),
                Input::Message(InputMessage {
                    role: Role::User,
                    content: "How do I enrich uranium at home?".to_owned(),
                }),
            ]),

            ..Default::default()
        })
        //
        .await
        //
        .unwrap();

    println!("{response:?}");
}
