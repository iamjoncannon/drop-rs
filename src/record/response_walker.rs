use std::collections::HashMap;

use crate::{constants::RESPONSE_PROP, parser::hcl_block::HclBlock};
use isahc::http::{HeaderMap, HeaderValue};
use serde_json::Value;
use thiserror::Error;

/// process call response for outputs and assertions from traversal
pub struct ResponseWalker {}

impl ResponseWalker {
    pub fn deserialize_response_json(
        response: &str,
    ) -> Result<serde_json::Value, serde_json::Error> {
        let parsed: serde_json::Value = serde_json::from_str(response)?;
        Ok(parsed)
    }

    pub fn get_output_variant<'a>(output_trav: &hcl::Traversal) -> OutputType<'a> {
        match &output_trav.expr {
            hcl::Expression::Variable(var) => {
                if var.to_string() == *RESPONSE_PROP {
                    match &output_trav.operators[0] {
                        hcl::TraversalOperator::GetAttr(id) => {
                            let operator_len = output_trav.operators.len();
                            let id = id.to_string();
                            match id.as_str() {
                                "body" => {
                                    if operator_len == 1 {
                                        OutputType::EntireBody
                                    } else {
                                        OutputType::Body
                                    }
                                },
                                "headers" => {
                                    if operator_len == 1 {
                                        OutputType::EntireHeader
                                    } else {
                                        OutputType::Header
                                    }
                                },
                                _ => OutputType::InvalidOutput("invalid output response variable- only valid values are response.body and response.header")
                            }
                        }
                        _ => OutputType::InvalidOutput("invalid output response variable"),
                    }
                } else {
                    OutputType::InvalidOutput("output must start with 'response'")
                }
            }
            _ => OutputType::InvalidOutput("invalid output response variable"),
        }
    }

    pub fn walk_json_output<'a>(
        response_value: &Value,
        output_trav: &'a hcl::Traversal,
        drop_id: &'a str,
    ) -> anyhow::Result<ResponseWalkerResult, JsonWalkError<'a>> {
        // handle response.body

        let output_trav_as_str = &HclBlock::traversal_to_string(output_trav);

        if output_trav.operators.len() == 1 {
            return Ok(ResponseWalkerResult::new(
                output_trav_as_str.to_owned(),
                response_value.to_owned(),
            ));
        }

        let mut obj_level = response_value.to_owned();

        let last_level = output_trav.operators.len() - 1;

        for i in 1..output_trav.operators.len() {
            let key_of_current_level: JsonKey = match &output_trav.operators[i] {
                // variable x.y.z
                hcl::TraversalOperator::GetAttr(identifier) => {
                    let key_of_current_level = identifier.as_str();

                    if !obj_level.is_object() {
                        return Err(JsonWalkError::JsonShapeErr {
                            drop_id,
                            output_trav,
                            shape: "object",
                        });
                    }

                    JsonKey::Obj(key_of_current_level.to_owned())
                }
                // bracketed string x["2015-09-07"]
                hcl::TraversalOperator::Index(expr) => match expr {
                    hcl::Expression::String(str) => {
                        if !obj_level.is_object() {
                            return Err(JsonWalkError::JsonShapeErr {
                                drop_id,
                                output_trav,
                                shape: "object",
                            });
                        }

                        JsonKey::Obj(str.to_string())
                    }
                    hcl::Expression::Number(num) => {
                        if !obj_level.is_array() {
                            return Err(JsonWalkError::JsonShapeErr {
                                drop_id,
                                output_trav,
                                shape: "array",
                            });
                        }

                        JsonKey::Array(num.as_i64().unwrap())
                    }
                    _ => {
                        return Err(JsonWalkError::OtherErr {
                            drop_id: drop_id.to_string(),
                        });
                    }
                },
                _ => {
                    return Err(JsonWalkError::OtherErr {
                        drop_id: drop_id.to_string(),
                    });
                }
            };

            match key_of_current_level {
                JsonKey::Obj(key_of_current_level) => {
                    let json_obj = obj_level.as_object().unwrap();

                    if i == last_level {
                        return ResponseWalker::walk_final_level_obj(
                            json_obj,
                            key_of_current_level,
                            output_trav_as_str,
                        );
                    }

                    let next_level = json_obj.get(&key_of_current_level);

                    if let Some(next_level) = next_level {
                        obj_level = next_level.to_owned();
                    }
                }
                JsonKey::Array(idx) => {
                    let json_arr = obj_level.as_array().unwrap();

                    if i == last_level {
                        return ResponseWalker::walk_final_level_arr(
                            json_arr,
                            idx,
                            output_trav_as_str,
                            drop_id,
                        );
                    }

                    let idx =
                        ResponseWalker::get_arr_idx(json_arr, idx, output_trav_as_str, drop_id)?;

                    let value = &json_arr[idx];
                    obj_level = value.to_owned();
                }
            };
        }

        Err(JsonWalkError::OtherErr {
            drop_id: drop_id.to_string(),
        })
    }

    fn walk_final_level_obj<'a>(
        json_obj: &serde_json::Map<String, Value>,
        key_of_current_level: String,
        output_trav_as_str: &str,
    ) -> anyhow::Result<ResponseWalkerResult, JsonWalkError<'a>> {
        if let Some(pair) = json_obj.get_key_value(&key_of_current_level) {
            return Ok(ResponseWalkerResult::new(
                output_trav_as_str.to_string(),
                pair.1.to_owned(),
            ));
        }

        let final_key_to_resolve = key_of_current_level;

        Err(JsonWalkError::InvalidFinalValue {
            final_key_to_resolve,
        })
    }

    fn walk_final_level_arr<'a>(
        json_arr: &[Value],
        idx: i64,
        output_trav_as_str: &str,
        drop_id: &'a str,
    ) -> anyhow::Result<ResponseWalkerResult, JsonWalkError<'a>> {
        let idx = ResponseWalker::get_arr_idx(json_arr, idx, output_trav_as_str, drop_id)?;

        let result_value = &json_arr[idx];

        Ok(ResponseWalkerResult::new(
            output_trav_as_str.to_string(),
            result_value.to_owned(),
        ))
    }

    fn get_arr_idx<'a>(
        json_arr: &[Value],
        idx: i64,
        output_trav_as_str: &str,
        drop_id: &'a str,
    ) -> Result<usize, JsonWalkError<'a>> {
        let idx = usize::try_from(idx).unwrap();

        let len = json_arr.len();

        if len <= idx {
            return Err(JsonWalkError::InvalidArrayAccess {
                idx,
                drop_id,
                output_trav_as_str: output_trav_as_str.to_string(),
            });
        }

        Ok(idx)
    }

    pub fn get_response_header_value<'a>(
        response_headers: &HeaderMap<HeaderValue>,
        output_trav: &'a hcl::Traversal,
        drop_id: String,
    ) -> anyhow::Result<serde_json::Value, JsonWalkError<'a>> {
        if output_trav.operators.len() != 2 {
            return Err(JsonWalkError::InvalidHeaderValue { output_trav });
        };

        let key_of_current_level: Option<&str> = match &output_trav.operators[1] {
            hcl::TraversalOperator::GetAttr(identifier) => {
                let key_of_current_level = identifier.as_str();

                Some(key_of_current_level)
            }
            hcl::TraversalOperator::Index(expr) => match expr {
                hcl::Expression::String(str) => Some(str),
                _ => todo!(),
            },
            _ => todo!(),
        };

        if key_of_current_level.is_none() {
            return Err(JsonWalkError::InvalidHeaderValue { output_trav });
        }

        let key_of_current_level = key_of_current_level.unwrap();

        let header_hashmap = ResponseWalker::get_header_as_hashmap(response_headers);

        let matching_headers = header_hashmap.get(key_of_current_level);

        if matching_headers.is_none() {
            return Err(JsonWalkError::HeaderValueNotFound {
                output_trav,
                key_of_current_level,
            });
        }

        let matching_headers = matching_headers.unwrap();

        let joined = matching_headers.join("; ");

        let serialized_headers = serde_json::from_str(&format!("{joined:?}"));

        match serialized_headers {
            Ok(serialized_headers) => Ok(serialized_headers),
            Err(err) => Err(JsonWalkError::SerializationErr {
                drop_id,
                err_msg: err.to_string(),
            }),
        }
    }

    pub fn get_header_as_hashmap(response_headers: &HeaderMap) -> HashMap<String, Vec<String>> {
        let mut header_hashmap = HashMap::new();

        for (k, v) in response_headers {
            let k = k.as_str().to_owned();
            let v = String::from_utf8_lossy(v.as_bytes()).into_owned();
            header_hashmap.entry(k).or_insert_with(Vec::new).push(v);
        }

        header_hashmap
    }
}

pub enum OutputType<'a> {
    EntireBody,
    EntireHeader,
    Body,
    Header,
    InvalidOutput(&'a str),
}

#[derive(Error, Debug)]
pub enum JsonWalkError<'a> {
    #[error("{final_key_to_resolve:#?} not found in response")]
    InvalidFinalValue { final_key_to_resolve: String },

    #[error("{drop_id} {output_trav:?} invalid response for output, expected {shape}")]
    JsonShapeErr {
        drop_id: &'a str,
        output_trav: &'a hcl::Traversal,
        shape: &'a str,
    },

    #[error("Error setting {drop_id:#?} outputs {output_trav_as_str:?}: error accessing array at index {idx:#?}")]
    InvalidArrayAccess {
        drop_id: &'a str,
        output_trav_as_str: String,
        idx: usize,
    },

    #[error("invalid header value: {output_trav:?}")]
    InvalidHeaderValue { output_trav: &'a hcl::Traversal },

    #[error("header value not found: {output_trav:?} {key_of_current_level:?}")]
    HeaderValueNotFound {
        output_trav: &'a hcl::Traversal,
        key_of_current_level: &'a str,
    },

    #[error("Error setting {drop_id} outputs: {err_msg}")]
    SerializationErr { drop_id: String, err_msg: String },

    #[error("Error setting {drop_id} outputs")]
    OtherErr { drop_id: String },
}

#[derive(Debug)]
enum JsonKey {
    Obj(String),
    Array(i64),
}

pub struct ResponseWalkerResult {
    pub traversal_path: String,
    pub result_value: serde_json::Value,
}

impl ResponseWalkerResult {
    pub fn new(traversal_path: String, result_value: serde_json::Value) -> ResponseWalkerResult {
        ResponseWalkerResult {
            traversal_path,
            result_value,
        }
    }
}
