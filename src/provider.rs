use crate::{error::Result, types::{CreateResponse, Output}};
use std::future::Future;

pub trait Provider: Send + Sync {
    type Config: Send + Sync + Clone;
    
    fn create_response(&self, request: &CreateResponse) -> impl Future<Output = Result<Vec<Output>>> + Send;
    
    fn name(&self) -> &'static str;
}

pub trait ProviderBuilder<P: Provider> {
    type Error;
    
    fn build(self) -> std::result::Result<P, Self::Error>;
}