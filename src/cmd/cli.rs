//! Command line interface for drop
use clap::{Parser, Subcommand, ValueEnum};
use std::error::Error;

#[derive(Parser, Debug, PartialEq)]
#[command(version, about)]
// #[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// log level
    #[arg(short, long, value_enum, default_value="info")]
    pub level: LogLevelInput,   

    /// drop environment
    #[arg(short, long, default_value="base")]
    pub env: String,

    /// dropfile directory
    #[arg(short, long, default_value=".")]
    pub dir: String,

    // inputs
    // #[arg(short = 'i', long, value_parser = parse_key_val::<String, String>)]
    // pub input: Option<Vec<(String, String)>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevelInput {
    Info,
    Debug,
    Trace
}

#[derive(Subcommand, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Command {

    ///
    /// run a specific call, defined by the call id.
    ///
    /// e.g.- a hit in env public
    ///
    /// `get "nasa_neos" {  
    ///     base_url = "https://api.nasa.gov"
    ///     path = "/neo/rest/v1/feed"
    /// }`
    ///
    /// to call- `drop hit public.get.nasa_neos``
    hit {
        /// either a module or the id of the call block to run
        drop_id: String,
    },

    ///
    /// evaluate drop in env before running
    ///
    give {
        /// the id of the call block to evaluate
        drop_id: String,
    },

    ///
    /// get and set secrets for environment
    ///
    secret {
        /// get or set
        action: String,

        /// key for secret
        key: Option<String>,

        /// value for secret
        value: Option<String>,
    },
}

/// Parse a single key-value pair
/// `<https://github.com/clap-rs/clap/blob/master/examples/typed-derive.rs />`
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}