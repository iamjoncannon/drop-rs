use std::time::Duration;
use anyhow::anyhow;
use isahc::{config::Configurable, http::{request::Builder, Error, HeaderMap}, HttpClient, ReadResponseExt, Request};

use crate::{assert::{assertion::CallAssertion, types::Assert}, call::DropCall, record::CallRecord};

/// http transaction manager
pub struct Caller {
    pub drop_call: DropCall
}

impl Caller {
    pub fn call(self) -> Result<CallRecord, anyhow::Error> {

        let client = HttpClient::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        let request_builder = self.generate_request_from_call();

        let response_result = if let Some(body) = &self.drop_call.body {
            let request = request_builder.body(serde_json::to_vec(body)?).unwrap();
            log::debug!("Caller request: {request:?}");
            println!("\ncalling {}", request.uri());
            client.send(request)
        
        } else {
            let request = request_builder.body(()).unwrap();
            log::debug!("Caller request: {request:?}");
            println!("calling {}", request.uri());
            client.send(request)
        };

        if response_result.is_err() {
            return self.handle_request_error(response_result.unwrap_err());
        } else {
            return self.handle_request_success(response_result.unwrap());
        }
    }

    pub fn generate_request_from_call(&self) -> Builder {
    
        let headers = &self.drop_call.headers;
        let full_url = self.drop_call.base_url.to_string() + &self.drop_call.path;
        let mut request = Request::builder().method(&self.drop_call.method).uri(full_url);

        for (k,v) in headers {
            request = request.header(k,v);
        }

        request
    }

    pub fn handle_request_error(&self, response_result: isahc::Error)-> Result<CallRecord, anyhow::Error> {
        // print error
        log::error!("error completing request {response_result}");

        Err(anyhow!("error completing request"))
    }

    pub fn handle_request_success(mut self, mut response: isahc::Response<isahc::Body>) -> Result<CallRecord, anyhow::Error> {

        log::debug!("response {response:?}");

        let response_string = &response.text();

        let drop_id_string = &self.drop_call.drop_id.drop_id().unwrap();
        let outputs = &self.drop_call.outputs.take();
        let asserts: Vec<Assert> = self.drop_call.asserts.drain(..).collect();

        let response_status = response.status();

        let response_headers = response.headers();

        let is_successful_call = response_status.is_success();

        if !is_successful_call {
            self.handle_failed_status_code(response_string);
        }

        let mut record = CallRecord::init(self.drop_call, is_successful_call);

        record.set_status_code(response_status);

        match response_string {
            Ok(response_string) => {

                log::debug!("response_string {response_string}");

                record.process_output_from_response(
                    outputs,
                    &response_string,
                    &response_headers,
                    is_successful_call,
                );

                if !asserts.is_empty() {
                    CallAssertion::run_assertions(
                        drop_id_string,
                        asserts,
                        &response_string,
                        &response_headers,
                    );
                }

            }
            _ => {
                log::error!("failure deserializing result")
            },
        }

        Ok(record)
    }

    fn handle_failed_status_code(&self, response_string: &Result<String, std::io::Error>) {
        match &response_string {
            Ok(msg) => {
                if msg.is_empty() {
                    log::trace!("raw response_string {response_string:#?}");
                    println!("\ncall failed (no response message)");
                } else {
                    let try_deserialize: Result<hcl::Value, serde_json::Error> =
                        serde_json::from_str(msg);

                    match try_deserialize {
                        Ok(ser) => {
                            println!("\ncall failed: {ser:?}");
                        }
                        _ => {
                            println!("\ncall failed: {msg:?}");
                        }
                    }
                }
            }
            Err(err) => {
                log::trace!("response_string unwrap err {err:#?}");
                println!("\ncall failed");
            }
        }
    }


}
