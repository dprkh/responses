use std::future::Future;

use crate::types::{CreateResponse, Output};

use anyhow::Result;

pub trait Strategy {
    fn create_response(
        //
        &self,
        //
        create_response: &CreateResponse,
    ) -> impl Future<Output = Result<Vec<Output>>>;
}
