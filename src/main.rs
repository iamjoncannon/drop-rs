#![allow(warnings)]

use clap::Parser;
use cmd::{cli::Cli, CmdContext};
use colored::Colorize;
use interpreter::scope::{GlobalScopeProvider, Scope};
use log::{error, LevelFilter};
use parser::{file_walker::FileWalker, GlobalDropConfig, GlobalDropConfigProvider};
use persist::{sqlite_persister::SqlitePersister, Persister};
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

// TODO
//  - create mock env, mod, and calls
//
// - parse current files and generate GlobalDropConfig
// - GlobalDropConfig propagated as global provider
// global drop config should be structured to service give, list, and hit
//
// - run command
// use log_derive::{logfn, logfn_inputs};
#[tokio::main]
async fn main() {
    // prevent panic from printing generic rust message
    std::panic::set_hook(Box::new(|err| {
        let entire_error = err.to_string();

        println!("\n\n{entire_error}\n\n");

        let exited = "\ndrop exited on error\n".red();
        let run_with_log = "run with DROP_LOG=debug for more info.".red();

        println!("{exited}{run_with_log}");
    }));

    // todo-- configure global log level from cli flag

    let mut log_config = ConfigBuilder::new();
    log_config.set_time_level(LevelFilter::Debug);

    let cli = Cli::parse();

    let level_filter = match cli.level {
            cmd::cli::LogLevelInput::Info => LevelFilter::Info,
            cmd::cli::LogLevelInput::Debug => LevelFilter::Debug,
            cmd::cli::LogLevelInput::Trace => LevelFilter::Trace,
    };

    TermLogger::init(
        // LevelFilter::Info,
        level_filter,
        // Config::default(),
        log_config.build(),
        TerminalMode::default(),
        ColorChoice::Always,
    )
    .unwrap();

    log::debug!("cli input {:?}", cli);

    let current_command = &cli.command;

    log::debug!("current_command {:?}", current_command);

    // TODO-- cli.dir
    let dropfile_dir = ".";

    let user_selected_env = "local";

    let input_drop_id = "example.get.example_hit";

    let mut persister = SqlitePersister::init();

    set_global_config(dropfile_dir);

    set_variable_scope(user_selected_env, &mut persister);

    // CmdContext should convert input_drop_id
    // to drop_id stuct
    let cmd = CmdContext { input_drop_id };

    CmdContext::set(cmd);

    log::info!(
        "hitting {} in environment {}\n",
        input_drop_id.yellow(),
        cli.env.unwrap().yellow()
    );

    // let target_drop_id = CmdContext::get_target_id();

    // let (hcl_block, drop_block, _eval_diag) =
    //     Evaluator::evaluate_call(target_drop_id.unwrap()).unwrap();

    // let drop_call =
    //     DropCall::from_hcl_block(&hcl_block, drop_block.drop_id.as_ref().unwrap().clone());

    // RunPool::runner_pool().await;
}

fn set_global_config(dropfile_dir: &str) {
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

fn set_variable_scope(user_selected_env: &str, persister: &mut SqlitePersister) {
    let secrets_hash_for_env_res = persister.get_secrets_for_env(user_selected_env, false);

    if secrets_hash_for_env_res.is_err() {
        error!(
            "Error resolving variable scope: {:?}",
            secrets_hash_for_env_res.unwrap_err()
        );
        std::process::exit(1)
    } else {
        let variable_context_res =
            Scope::evaluate_variable_scope(secrets_hash_for_env_res.unwrap(), user_selected_env);

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
