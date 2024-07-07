use std::vec;

use hcl::{Attribute, Block, Expression, ObjectKey};
use isahc::http::{HeaderName, HeaderValue};
use serde_json::Value;

use crate::{constants::CALL_BODY, parser::hcl_block::HclBlock};
use log::{error, warn};

use super::DropCall;

impl DropCall {
    pub fn process_headers(&mut self, attr: &Attribute) {
        let exp = attr.expr();

        if let Expression::Array(headers_vec) = exp {
            for header in headers_vec {
                if let Expression::Object(vec_map) = header {
                    for (key, value) in vec_map {
                        self.process_header(key, value);
                    }
                }
            }
        } else {
            panic!("headers must be an array of objects")
        }
    }

    pub fn process_header(&mut self, key: &ObjectKey, value: &Expression) {
        let raw_name = key.to_string();

        let header_name = HeaderName::from_bytes(raw_name.as_bytes());

        match header_name {
            Ok(name) => {
                let raw_value = HclBlock::format_hcl_raw_string(value.to_string());
                let header_value = HeaderValue::from_str(&raw_value).unwrap();

                self.headers.insert(name, header_value);
            }
            Err(err) => warn!("process_headers HeaderName err {err:#?} raw_name {raw_name:#?}"),
        }
    }

    pub fn process_output_config(&mut self, attr: &Attribute) {
        let exp = attr.expr();

        if let Expression::Array(output_key_vec) = exp {
            for output_key in output_key_vec {
                if let Expression::Traversal(trav) = output_key {
                    if let Some(current_output_vec) = &mut self.outputs {
                        current_output_vec.push(*trav.to_owned());
                    } else {
                        self.outputs = Some(vec![*trav.to_owned()]);
                    }
                } else {
                    let drop_id = self.drop_id.drop_id().unwrap();
                    error!("{drop_id:#?}: each output must be a variable ");
                    std::process::exit(1)
                }
            }
        } else {
            let drop_id = self.drop_id.drop_id().unwrap();
            error!("{drop_id:#?}: outputs must be an array of variables");
            std::process::exit(1)
        }
    }

    ///
    /// hcl-rs can only transform valid hcl blocks to the json format
    /// expected by reqwest `serde::Value`
    ///
    /// serialize the entire call block and then pick out the body property
    ///
    pub fn process_body(&mut self, full_call_block: &Block, method: &str) {
        let resource_name = &self.drop_id.resource_name;

        let serialized = hcl::to_string(&full_call_block).unwrap();

        let hcl_as_serde_value: Result<Value, hcl::Error> = hcl::from_str(&serialized);

        if hcl_as_serde_value.is_err() {
            let msg = hcl_as_serde_value.err();
            error!("{resource_name:#?}: request body must be a json object {msg:#?}");
            std::process::exit(1)
        }

        let valid_hcl_as_serde_value = hcl_as_serde_value.unwrap();

        // if we're here then we know the obejct path will be method.[resource_name].body
        let body_serde_value = valid_hcl_as_serde_value
            .get(method)
            .unwrap()
            .get(resource_name)
            .unwrap()
            .get(CALL_BODY)
            .unwrap();

        self.set_body(body_serde_value);
    }
}
