use hcl::{Attribute, Expression};
use log::error;

use crate::{action::PostAction, constants::*};

use super::DropCall;

impl DropCall {

    pub fn process_afters(&mut self, attr: &Attribute) {

        // todo- refactor into Result pattern
        let drop_id = &self.drop_id.drop_id().unwrap().to_owned();

        let exp = attr.expr();

        if let Expression::Array(after_objects) = exp {
            for after_object in after_objects {
                self.process_after(after_object, drop_id);
            }
        } else {
            error!("{drop_id:#?}: after must be an array of after action objects");
            panic!()
        }
    }

    pub fn process_after(&mut self, after_object: &Expression, drop_id: &String) {
        if let Expression::Object(obj) = after_object {
            let get_action_id = || {
                for (object_key, exp_val) in obj {
                    if object_key.to_string() == TYPE_PROP {
                        return exp_val.to_string();
                    }
                }

                DropCall::after_throw(drop_id);
                panic!()
            };

            let _action_id = get_action_id();
            let _set = AFTER_SET_SECRET_TYPE_VALUE.to_string();

            let action_hash = PostAction::prepare_action_from_hcl_expression(after_object);

            self.after_action_config_push(drop_id, action_hash);
        } else {
            DropCall::after_throw(drop_id);
            panic!()
        }
    }

    pub fn after_throw(drop_id: &String) {
        error!("{drop_id:#?}: invalid after action objects");
    }
}
