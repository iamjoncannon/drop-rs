use derive_getters::Getters;
use hcl::Traversal;
use isahc::http::{HeaderMap, HeaderValue, StatusCode};
use log::{error, trace};
use output_record::OutputRecord;
use response_walker::{OutputType, ResponseWalker};

use crate::{action::AfterActionConfig, call::DropCall, parser::hcl_block::HclBlock, s};

mod output_record;
pub mod response_walker;
#[derive(Clone, Debug, Getters)]
pub struct CallRecord {
    pub drop_id: String,
    pub full_url: String,
    pub status_code: Option<StatusCode>,
    pub full_response: Option<String>,
    pub output_records: Option<Vec<OutputRecord>>,
    pub after_action_config: Option<AfterActionConfig>,
    pub is_successful_call: bool,
}

impl CallRecord {
    pub fn init(call: DropCall, is_successful_call: bool) -> CallRecord {
        CallRecord {
            drop_id: call.drop_id.drop_id().unwrap(),
            full_url: call.full_url(),
            after_action_config: Some(call.after_action_config),
            status_code: None,
            full_response: None,
            output_records: None,
            is_successful_call
        }
    }

    pub fn set_status_code(&mut self, status_code: StatusCode) {
        self.status_code = Some(status_code);
    }

    pub fn process_output_from_response(
        &mut self,
        outputs: &Option<Vec<Traversal>>,
        response_body: &String,
        response_headers: &HeaderMap<HeaderValue>,
        is_successful_call: bool,
    ) {
        self.full_response = Some(response_body.to_owned());

        let response_body_as_possible_json =
            ResponseWalker::deserialize_response_json(response_body);

        match &outputs {
            Some(outputs) => {
                for output_trav in outputs {
                    let output_variant = ResponseWalker::get_output_variant(output_trav);
                    let output_trav_as_str = HclBlock::traversal_to_string(output_trav);

                    match output_variant {
                        OutputType::EntireBody => {
                            match &response_body_as_possible_json {
                                Ok(response_as_possible_json) => {
                                    self.set_output(&output_trav_as_str, response_as_possible_json);
                                }
                                Err(err) => {
                                    if err.to_string() == "expected value at line 1 column 1" {
                                        // not json
                                        self.set_output(
                                            &output_trav_as_str,
                                            &serde_json::Value::String(response_body.to_string()),
                                        );
                                    } else {
                                        log::debug!("process_output_from_response error setting outputs {err}");
                                        log::error!("error setting outputs {err}");
                                    }
                                }
                            }
                        }
                        OutputType::Body => {
                            self.get_response_body_value(
                                &self.drop_id().to_owned(),
                                output_trav,
                                &response_body_as_possible_json,
                                is_successful_call,
                            );
                        }
                        OutputType::EntireHeader => {
                            let header_hashmap =
                                ResponseWalker::get_header_as_hashmap(response_headers);

                            let serialized_headers: Result<serde_json::Value, serde_json::Error> =
                                serde_json::from_str(&format!("{header_hashmap:?}"));

                            match serialized_headers {
                                Ok(serialized_headers) => {
                                    self.set_output(&output_trav_as_str, &serialized_headers);
                                }
                                Err(err) => {
                                    eprintln!("error deserializing response header {response_headers:?}: {}", s!(err));
                                }
                            }
                        }
                        OutputType::Header => {
                            self.get_response_header_value(response_headers, output_trav);
                        }
                        OutputType::InvalidOutput(err) => {
                            eprintln!("{err}");
                        }
                    }
                }
            }
            None => {}
        }
    }

    /// access response body and set value
    /// from target traversal in output config
    fn get_response_body_value(
        &mut self,
        drop_id: &String,
        output_trav: &Traversal,
        response_as_possible_json: &Result<serde_json::Value, serde_json::Error>,
        is_successful_call: bool,
    ) {
        trace!("response_as_possible_json {response_as_possible_json:?}");
        match response_as_possible_json {
            Ok(response_value) => {
                let walk_result =
                    ResponseWalker::walk_json_output(response_value, output_trav, drop_id);

                match walk_result {
                    Ok(res) => {
                        self.set_output(&res.traversal_path, &res.result_value);
                    }
                    Err(walk_err) => {
                        if is_successful_call {
                            eprintln!("{walk_err}");
                        } else {
                            trace!("{walk_err}");
                        }
                    }
                }
            }
            Err(err) => {
                trace!("get_response_body_value deserializaiton error {err}");
                error!("error setting {drop_id:#?} outputs {:?}", HclBlock::traversal_to_string(output_trav));
            }
        }
    }

    fn get_response_header_value(
        &mut self,
        response_headers: &HeaderMap<HeaderValue>,
        output_trav: &hcl::Traversal,
    ) {
        let walk_result = ResponseWalker::get_response_header_value(
            response_headers,
            output_trav,
            self.drop_id().to_string(),
        );

        if let Ok(value) = &walk_result {
            self.set_output(&HclBlock::traversal_to_string(output_trav), value);
        } else {
            eprintln!("{:?}", walk_result.err());
        }
    }

    pub fn set_output(&mut self, key: &String, value: &serde_json::Value) {
        let record = match value {
            serde_json::Value::String(value) => OutputRecord {
                key: key.to_string(),
                value: value.clone(),
            },
            _ => OutputRecord {
                key: key.to_string(),
                value: value.to_string(),
            },
        };

        trace!("setting record {record:?}");

        match &self.output_records {
            Some(current_output_records) => {
                let mut next_outputs = current_output_records.clone();
                next_outputs.push(record);
                self.output_records = Some(next_outputs);
            }
            None => {
                self.output_records = Some(vec![record]);
            }
        }
    }
}
