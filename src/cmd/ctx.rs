use std::sync::OnceLock;
use anyhow::anyhow;

use super::cli::{Cli, Command};

static GLOBAL_CMD_CTX_PROVIDER: OnceLock<CmdContext> = OnceLock::new();

/// global context provider for clap cli input
#[derive(Debug)]
pub struct CmdContext{
    cli: Cli,
}

impl CmdContext {

    pub fn set(cli: Cli) {

        log::debug!("CmdContext set {cli:?}");

        let cmd = CmdContext { cli };

        let cell_set_result = GLOBAL_CMD_CTX_PROVIDER.set(cmd);

        if cell_set_result.is_err() {
            log::error!(
                "Error setting global cmd context provider: {:?}",
                cell_set_result.unwrap_err()
            )
        }
    }

    pub fn get() -> Result<&'static CmdContext, anyhow::Error> {
        let res = GLOBAL_CMD_CTX_PROVIDER.get();

        if let Some(ctx) = res {
            Ok(ctx)
        } else {
            Err(anyhow!("error accessing global cmd ctx"))
        }
    }

    pub fn get_env() -> &'static str {

        let cmd = CmdContext::get();

        if cmd.is_err() {
            log::warn!("error unwrapping CmdContext {:?}", cmd.unwrap_err());

            "base"
        } else {
            &cmd.unwrap().cli.env
        }
    }
}
