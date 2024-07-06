use serde::{Deserialize, Serialize};
use anyhow::anyhow;
use super::drop_block::DropResourceType;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct DropId {
    pub module: Option<String>,
    pub resource_type: DropResourceType,
    call_method: Option<String>,
    pub resource_name: String,
}

pub enum CallType {
    Hit,
    Run,
    // Chain,
}

impl DropId {
    pub fn new(
        module: Option<String>,
        resource_type: DropResourceType,
        call_method: Option<String>,
        resource_name: &str,
    ) -> DropId {
        DropId {
            module,
            resource_type,
            call_method,
            resource_name: resource_name.to_string(),
        }
    }

    pub fn get_call_drop_id(method: &str, module_declaration: &str, block_title: &str) -> DropId {
        DropId::new(
            Some(module_declaration.to_string()),
            DropResourceType::Call,
            Some(method.to_string()),
            block_title,
        )
    }

    // returns entire drop id string
    pub fn drop_id(&self) -> Result<String, anyhow::Error> {
        match &self.module {
            Some(_module) => {
                let module = self.module.as_deref().unwrap().to_string();

                match &self.call_method {
                    Some(_call_method) => {
                        let call_method = self.call_method.as_deref().unwrap().to_string();
                        Ok(format!("{}.{}.{}", module, call_method, self.resource_name))
                    }
                    _ => Ok(format!("{}.{}", module, self.resource_name)),
                }
            }

            _ => Err(anyhow!("failed to generate drop id for {:?}", self)),
        }
    }

    pub fn _is_drop_id(id: &str) -> bool {
        let split: Vec<&str> = id.split('.').collect();

        split.len() == 3
    }

    pub fn get_module_from_drop_id(drop_id_str: &str) -> &str {
        let split: Vec<&str> = drop_id_str.split('.').collect();

        assert!(split.len() == 3, "invalid drop id passed: {}\nDropid has pattern mod.method.label, e.g. `public.get.nasa_neos`.", drop_id_str);

        split[0]
    }

    pub fn get_call_type_from_raw_drop_id(drop_id_str: &str) -> CallType {
        let split: Vec<&str> = drop_id_str.split('.').collect();

        assert!(split.len() == 3, "invalid drop id passed: {}\nDropid has pattern mod.method.label, e.g. `public.get.nasa_neos`.", drop_id_str);

        let call = split[1];

        let all_calls = ["get", "post", "put", "patch", "delete", "chain", "run"];

        assert!(
            all_calls.contains(&call),
            "invalid drop id passed: {}\nValid call drop ids are {}.",
            drop_id_str,
            all_calls.join(" ")
        );

        match call {
            // "chain" => CallType::Chain,
            "run" => CallType::Run,
            "get" | "post" | "delete" | "put" | "patch" => CallType::Hit,
            _ => panic!(),
        }
    }

    pub fn get_resource_type_from_drop_id(drop_id_str: &str) -> DropResourceType {

        let call_type = DropId::get_call_type_from_raw_drop_id(drop_id_str);

        match call_type {
            CallType::Hit => DropResourceType::Call,
            CallType::Run => todo!(),
            // CallType::Chain => todo!(),
        }
    }
}
