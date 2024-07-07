use crate::{cmd::{ctx::CmdContext, DropCommand}, exit, parser::GlobalDropConfigProvider, persist::{Persister, PersisterProvider}};
use colored::Colorize;
use std::{io, process};

/// manages getting setting secrets in environment
#[derive(Debug)]
pub struct SecretCommand {
    pub action: String,
    pub key: Option<String>,
    pub value: Option<String>,
}

impl DropCommand for SecretCommand {
    fn announce(&self) {
        // todo!()
    }

    fn run(&mut self) -> std::pin::Pin<Box<dyn futures::Future<Output = ()>>> {

        let env = CmdContext::get_env();

        let mut persister = PersisterProvider::get_lock_to_persister().unwrap();

        let action: &str = &self.action;
        let key = &self.key;
        let value = &self.value;

        match action {
            "del" => match key {
                Some(key) => persister.delete_secret_in_env(key, env),
                None => exit!("must set key to delete"),
            },
            "set" => {
                assert!(
                    !(key.is_none() || value.is_none()),
                    "must include both a key and value for secret to set"
                );
    
                let key = key.as_ref().unwrap();
                let value = value.as_ref().unwrap();
    
                println!("Please confirm setting secret:\n\nenvironment {}\nkey {}\nvalue {}\n\n'Y' or 'y' to proceed, any other key to cancel.", env.yellow(), key.yellow(), value.yellow());
    
                let stdin = io::stdin();
                let input = &mut String::new();
    
                loop {
                    input.clear();
    
                    let _ = stdin.read_line(input);
    
                    let input_val = (*input).to_string();
    
                    let matcher = |answer: &str| match answer {
                        "y" | "Y" => true,
                        _ => {
                            println!("\ncancelled");
                            process::exit(0)
                        }
                    };
    
                    let is_confirmed = matcher(input_val.trim());
    
                    if is_confirmed {
                        break;
                    }
                }
    
                persister.insert_secret_into_env(key, value, env, false);
            }
            "get" => {
                if env.is_empty() {
                    persister.get_all_secrets();
                } else {
                    persister.get_secrets_for_env(env, true);
                }
            }
            _ => {
                panic!("invalid action passed to secret: {action}. Only valid actions are get and set")
            }
        }

        Box::pin(async {})
    }
}