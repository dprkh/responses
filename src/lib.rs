
pub(crate) mod util;

pub mod types;


pub mod options;

pub mod response;

pub mod refusal;

pub mod schema;

pub mod error;

pub mod provider;

pub mod providers;

pub mod client;

pub mod builders;

pub mod request;


pub mod messages;

pub mod functions;

pub mod prompt;

pub mod judge;


pub use options::Options;

pub use response::Response;

pub use refusal::Refusal;

pub use error::{Error, Result};

pub use provider::Provider;

pub use client::Client;

pub use providers::*;

pub use builders::{azure, openai};

pub use messages::{Messages, messages};

// Re-export the macro
pub use responses_macros::tool;

// Re-export schemars to prevent version conflicts
// Users can now use `responses::schemars` instead of adding their own dependency
pub use schemars;

// Re-export judge types
pub use judge::{Judge, Judgment};

// Macros with #[macro_export] are automatically available at crate root
// Users can use: responses::assert_passes!, responses::assert_fails!, responses::assert_confidence!
