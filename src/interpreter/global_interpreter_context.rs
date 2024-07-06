use base64::{prelude::BASE64_STANDARD, Engine};
use hcl::{
    eval::{Context, FuncDef, ParamType},
    Value,
};
use indexmap::IndexMap;

pub struct GlobalInterpreterContext {}

impl GlobalInterpreterContext {
    pub fn create<'a>() -> Context<'a> {
        let mut variable_context = Context::new();
        variable_context.declare_func("join", GlobalInterpreterContext::join_hcl_fn());
        variable_context.declare_func("url_params", GlobalInterpreterContext::get_params_fn());
        variable_context.declare_func(
            "bearer_auth",
            GlobalInterpreterContext::bearer_auth_hcl_func(),
        );
        variable_context.declare_func(
            "basic_auth",
            GlobalInterpreterContext::basic_auth_hcl_func(),
        );
        variable_context.declare_func("base64", GlobalInterpreterContext::base64_hcl_func());
        variable_context.declare_var("inputs", IndexMap::new());
        variable_context.declare_var("secrets", IndexMap::new());
        variable_context
    }

    pub fn join_hcl_fn() -> FuncDef {
        FuncDef::builder()
            .variadic_param(ParamType::Any)
            .build(|args| {
                let mut concatted = Vec::<String>::new();

                for arg in args.iter() {
                    match arg {
                        Value::String(str) => {
                            concatted.push(str.to_string());
                        }
                        Value::Number(num) => {
                            concatted.push(num.to_string());
                        }
                        _ => panic!("only string or number can be concatted"),
                    }
                }

                Ok(Value::String(concatted.join("")))
            })
    }

    pub fn get_params_fn() -> FuncDef {
        FuncDef::builder()
            .variadic_param(ParamType::Array(Box::new(ParamType::String)))
            .build(|args| {
                let mut param_list: Vec<String> = Vec::new();

                for each in args.iter() {
                    if let Value::Array(arr) = each {
                        if arr.len() != 2 {
                            println!("params can only have two members- key and value");
                            panic!()
                        }

                        let this_string =
                            arr[0].as_str().unwrap().to_string() + "=" + arr[1].as_str().unwrap();

                        param_list.push(this_string);
                    }
                }

                let joined = "?".to_string() + &param_list.join("&");

                Ok(Value::String(joined))
            })
    }

    pub fn bearer_auth_hcl_func() -> FuncDef {
        FuncDef::builder().param(ParamType::String).build(|args| {
            let auth_string = "Bearer ".to_string() + args[0].as_str().unwrap();
            Ok(Value::String(auth_string))
        })
    }

    pub fn basic_auth_hcl_func() -> FuncDef {
        FuncDef::builder()
            .param(ParamType::String)
            .param(ParamType::String)
            .build(|args| {
                let name_and_password =
                    args[0].as_str().unwrap().to_owned() + ":" + args[1].as_str().unwrap();
                let mut buf = String::new();
                BASE64_STANDARD.encode_string(name_and_password, &mut buf);
                Ok(Value::String(buf))
            })
    }

    pub fn base64_hcl_func() -> FuncDef {
        FuncDef::builder()
            .param(ParamType::String)
            .build(|args| match &args[0] {
                Value::String(str) => {
                    let mut buf = String::new();

                    BASE64_STANDARD.encode_string(str, &mut buf);

                    Ok(Value::String(buf))
                }
                _ => {
                    panic!()
                }
            })
    }
}
