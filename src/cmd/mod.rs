use std::pin::Pin;

use cli::Command;
use commands::{give::GiveCommand, hit::HitCommand, secret::SecretCommand};
use futures::{future::BoxFuture, Future};


pub mod cli;
pub mod ctx;
pub mod commands;
pub mod dropdown;

pub trait DropCommand {
    fn announce(&self);
    fn run(&mut self) -> Pin<Box<dyn Future<Output = ()>>>;
}

/// converts clap cli command into
/// drop command interface
pub struct CommandManager {}

impl CommandManager {
    pub fn get_command(command: &Command) -> Box<dyn DropCommand> {

        log::debug!("command {:?}", command);

        match command {
            Command::hit { drop_id } => Box::new(HitCommand{ input_drop_id_string: drop_id.to_string() }),
            Command::give { drop_id } => Box::new(GiveCommand{ input_drop_id_string: drop_id.to_string() }),
            Command::secret { action, key, value } => Box::new(SecretCommand{ action: action.to_string(), key: key.to_owned(), value: value.to_owned() }),
        }
    }
}

