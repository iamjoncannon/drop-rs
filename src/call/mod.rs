use core::panic;
use std::collections::HashMap;

use derive_getters::Getters;
use hcl::{Attribute, Block, Traversal};
use indexmap::IndexMap;
use isahc::http::{HeaderMap, Method};

use crate::action::{ActionValue, AfterActionConfig};
use crate::assert::types::Assert;
use crate::constants::*;

use crate::parser::drop_id::DropId;
use crate::parser::hcl_block::HclBlock;

pub mod call_after;
pub mod call_assert;
pub mod call_auth;
pub mod call_inputs;
pub mod call_process;

/// main structure to manage api call
///
/// manages processing call values
/// from evaluated hcl block
#[derive(Debug)]
pub struct DropCall {
    pub drop_id: DropId,
    pub method: Method,
    pub base_url: String,
    pub path: String,
    pub headers: HeaderMap,
    pub body: Option<serde_json::Value>,
    pub inputs: Option<IndexMap<String, hcl::Value>>,
    pub outputs: Option<Vec<Traversal>>,
    pub after_action_config: AfterActionConfig,
    pub asserts: Vec<Assert>,
}

impl DropCall {
    pub fn full_url(&self) -> String {
        self.base_url.to_owned() + &self.path
    }

    pub fn set_body(&mut self, body: &serde_json::Value) {
        self.body = Some(body.to_owned());
    }

    pub fn set_null_outputs(&mut self) {
        self.outputs = None;
    }

    pub fn default(drop_id: DropId, block_type: &str) -> DropCall {
        DropCall {
            drop_id,
            method: DropCall::match_method_from_string(block_type),
            base_url: String::new(),
            path: String::new(),
            headers: HeaderMap::new(),
            body: None,
            outputs: None,
            inputs: None,
            after_action_config: HashMap::new(),
            asserts: Vec::<Assert>::new(),
        }
    }

    pub fn from_call_hcl_block(block: &Block, drop_id: DropId) -> DropCall {
        let block_type = block.identifier();
        let block_body = block.body();

        let mut call = DropCall::default(drop_id, block_type);

        for attr in block_body.attributes() {
            let as_raw_string = HclBlock::format_hcl_raw_string(attr.expr().to_string());
            match attr.key() {
                CALL_BASE_URL => call.base_url = as_raw_string,
                CALL_PATH => call.path = as_raw_string,
                CALL_HEADERS => call.process_headers(attr),
                CALL_OUTPUT => call.process_output_config(attr),
                CALL_AFTER => call.process_afters(attr),
                CALL_BODY => call.process_body(block, block_type),
                CALL_ASSERT => call.process_assert_block(attr),
                CALL_INPUTS => call.process_input_block(attr),
                _ => {
                    log::warn!("invalid attribute found on call block {:?}", attr.key())
                }
            }
        }
        call
    }

    pub fn from_call_and_run_hcl_block(
        call_block: &Block,
        run_block: &Block,
        drop_id: DropId,
    ) -> DropCall {
        let mut call = DropCall::from_call_hcl_block(call_block, drop_id);

        // remove assert, after action, and output blocks
        call.asserts = Vec::new();
        call.outputs = Some(Vec::new());
        call.after_action_config = HashMap::new();

        let get_attr = |key: &str|{
            let target_attr: Vec<&Attribute> = run_block
            .body()
            .attributes()
            .into_iter()
            .filter(|attr| attr.key() == key)
            .collect();

            target_attr
        };

        let assert_attr = get_attr(CALL_ASSERT);

        if !assert_attr.is_empty() {
            call.process_assert_block(assert_attr[0]);
        }

        let output_attr = get_attr(CALL_OUTPUT);

        if !output_attr.is_empty() {
            call.process_output_config(output_attr[0]);
        }

        let after_attr = get_attr(CALL_AFTER);

        if !after_attr.is_empty() {
            call.process_afters(after_attr[0]);
        }

        call
    }

    fn match_method_from_string(body_id: &str) -> Method {
        match body_id {
            GET_BLOCK_IDENTIFIER => Method::GET,
            POST_BLOCK_IDENTIFIER => Method::POST,
            _PUT_BLOCK_KEY => Method::PUT,
            PATCH_BLOCK_IDENTIFIER => Method::PATCH,
            _DELETE_BLOCK_KEY => Method::DELETE,
            _ => {
                panic!("Method not supported");
            }
        }
    }

    pub fn after_action_config_push(
        &mut self,
        drop_id: &String,
        action_hash: HashMap<String, ActionValue>,
    ) {
        if !self.after_action_config.contains_key(drop_id) {
            self.after_action_config
                .insert(drop_id.to_owned(), Vec::new());
        }

        let mut current_vec = self.after_action_config[drop_id].clone();

        current_vec.push(action_hash);

        self.after_action_config
            .insert(drop_id.to_owned(), current_vec);
    }
}
