use std::time::Duration;

use isahc::{config::Configurable, http::request::Builder, HttpClient, Request};

use crate::call::DropCall;

/// http transaction manager
pub struct Caller {
    drop_call: DropCall
}

impl Caller {
    pub fn call(self) -> Result<(), anyhow::Error> {

        let client = HttpClient::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        let request = Request::builder().method("POST");

        if let Some(body) = self.body {
            client.send(request.body(serde_json::to_vec(&body)?).unwrap());
        } else {
            client.send(request.body(()).unwrap());
        }


        Ok(())
    }

    pub fn generate_request_from_call(self) -> Builder {

        let request = Request::builder().method(&self.drop_call.method);

        request
    }
}
