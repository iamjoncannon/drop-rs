#![allow(warnings)]

use std::fmt;

use clap::Parser;
use cmd::{
    cli::{Cli, LogLevelInput},
    ctx::CmdContext, CommandManager,
};
use colored::Colorize;
use interpreter::scope::{GlobalScopeProvider, Scope};
use log::{error, LevelFilter};
use parser::{file_walker::FileWalker, GlobalDropConfig, GlobalDropConfigProvider};
use persist::{sqlite_persister::SqlitePersister, Persister, PersisterProvider};
use simplelog::{ColorChoice, Config, ConfigBuilder, TermLogger, TerminalMode};

mod action;
mod assert;
mod call;
mod caller;
mod cmd;
mod constants;
mod interpreter;
mod parser;
mod persist;
mod record;
mod runner;
mod util;

#[tokio::main]
async fn main() {
    setup_panic_handler();

    let cli = Cli::parse();

    setup_logger(cli.level);

    let command = &cli.command;

    setup_global_config(&cli.dir);

    setup_variable_scope(&cli.env);

    let mut drop_command = CommandManager::get_command(command);

    CmdContext::set(cli);

    drop_command.announce();

    drop_command.run().await;
}

fn setup_panic_handler() {
    // prevent panic from printing generic rust message
    std::panic::set_hook(Box::new(|err| {
        let entire_error = err.to_string();

        println!("\n\n{entire_error}\n\n");

        let exited = "\ndrop exited on error\n".red();
        let run_with_log = "run with log level flag ('-l debug') for more info.".red();

        println!("{exited}{run_with_log}");
    }));
}

fn setup_logger(level: LogLevelInput) {
    let mut log_config = ConfigBuilder::new();
    log_config.set_time_level(LevelFilter::Debug);

    let level_filter = match level {
        cmd::cli::LogLevelInput::Info => LevelFilter::Info,
        cmd::cli::LogLevelInput::Debug => LevelFilter::Debug,
        cmd::cli::LogLevelInput::Trace => LevelFilter::Trace,
    };

    TermLogger::init(
        level_filter,
        log_config.build(),
        TerminalMode::default(),
        ColorChoice::Always,
    )
    .unwrap();
}

fn setup_global_config(dropfile_dir: &str) {
    let resolve_drop_files_res = FileWalker::resolve_drop_files(dropfile_dir);

    if resolve_drop_files_res.is_err() {
        error!(
            "Error reading drop files: {}",
            resolve_drop_files_res.unwrap_err()
        );

        std::process::exit(1)
    } else {
        let global_drop_config_res =
            GlobalDropConfig::from_drop_files(&resolve_drop_files_res.unwrap());

        if global_drop_config_res.is_err() {
            error!(
                "Error resolving drop files: {:?}",
                global_drop_config_res.unwrap_err()
            );
            std::process::exit(1)
        } else {
            GlobalDropConfigProvider::set(global_drop_config_res.unwrap());
        }
    }
}

fn setup_variable_scope(user_selected_env: &str) {
    let persister_lock = PersisterProvider::get_lock_to_persister();

    if persister_lock.is_none() {
        log::warn!("setup_variable_scope failed to obtain lock to persister")
    } else {
        let secrets_hash_for_env_res = persister_lock
            .unwrap()
            .get_secrets_for_env(user_selected_env, false);

        if secrets_hash_for_env_res.is_err() {
            error!(
                "Error resolving variable scope: {:?}",
                secrets_hash_for_env_res.unwrap_err()
            );
            std::process::exit(1)
        } else {
            let variable_context_res = Scope::evaluate_variable_scope(
                secrets_hash_for_env_res.unwrap(),
                user_selected_env,
            );

            if variable_context_res.is_err() {
                error!(
                    "Error resolving variable scope: {:?}",
                    variable_context_res.unwrap_err()
                );
                std::process::exit(1)
            } else {
                GlobalScopeProvider::set(variable_context_res.unwrap());
            }
        }
    }
}
