use core::panic;
use std::collections::HashMap;

use derive_getters::Getters;
use hcl::{Block, Traversal};
use isahc::http::{HeaderMap, Method};

use crate::action::{ActionValue, AfterActionConfig};
use crate::assert::types::Assert;
use crate::constants::{
    CALL_AFTER, CALL_ASSERT, CALL_BASE_URL, CALL_BODY, CALL_HEADERS, CALL_OUTPUT, CALL_PATH, GET_BLOCK_IDENTIFIER, PATCH_BLOCK_IDENTIFIER, POST_BLOCK_IDENTIFIER
};
use crate::parser::drop_id::DropId;
use crate::parser::hcl_block::HclBlock;

pub mod call_after;
pub mod call_assert;
pub mod call_auth;
pub mod call_process;

/// main structure to manage api call
/// manages processing call values 
/// from evaluated hcl block
#[derive(Debug, Getters)]
pub struct DropCall {
    // pub drop_meta: DropMeta, // -> DropId
    pub drop_id: DropId,
    method: String,
    base_url: String,
    path: String,
    pub headers: HeaderMap,          
    body: Option<serde_json::Value>, 
    pub outputs: Option<Vec<Traversal>>,
    pub after_action_config: AfterActionConfig,
    pub asserts: Vec<Assert>,

    // move into Call Runner
    // pub result_mutex: Option<Arc<Mutex<HashMap<String, String>>>>,
    // pub channel: Option<(Receiver<String>, Sender<String>)>,
}

impl DropCall {
    pub fn full_url(&self) -> String {
        self.base_url().to_owned() + self.path()
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
            // drop_meta: DropMeta::new(drop_id.to_owned()),
            method: DropCall::match_method_from_string(block_type).to_string(),
            base_url: String::new(),
            path: String::new(),
            headers: HeaderMap::new(),
            body: None,
            outputs: None,
            after_action_config: HashMap::new(),
            // result_mutex: None,
            // channel: None,
            asserts: Vec::<Assert>::new(),
        }
    }

    pub fn from_hcl_block(block: &Block, drop_id: DropId) -> DropCall {
        let block_type = block.identifier();
        let _block_label = block.labels();
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
                _ => {}
            }
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

    // pub fn remove_result_mutex(&mut self) -> Option<Arc<Mutex<HashMap<String, String>>>> {
        // self.result_mutex.take()
    // }

    // pub fn remove_channel(&mut self) -> Option<(Receiver<String>, Sender<String>)> {
    //     self.channel.take()
    // }
}
