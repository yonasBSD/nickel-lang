use std::{
    fs, io,
    path::{self, PathBuf},
    thread,
};

use anyhow::Result;

use git_version::git_version;
use log::debug;
use lsp_server::Connection;

mod actions;
mod analysis;
mod background;
mod codespan_lsp;
mod command;
mod config;
mod diagnostic;
mod error;
mod field_walker;
mod files;
mod identifier;
mod incomplete;
mod position;
mod requests;
mod server;
use server::Server;
mod term;
mod trace;
mod usage;
mod world;

// Default stack size is 1MB on Windows, which is too small. We make it 8MB, which is the default
// size on Linux.
const STACK_SIZE: usize = 8 * 1024 * 1024;

use crate::{config::LspConfig, trace::Trace};

#[derive(clap::Parser, Debug)]
/// The language server of the Nickel language.
#[command(
    author,
    about,
    long_about = None,
    version = format!(
        "{} {} (rev {})",
        env!("CARGO_BIN_NAME"),
        env!("CARGO_PKG_VERSION"),
        // 7 is the length of self.shortRev. the string is padded out so it can
        // be searched for in the binary
        // The crate published on cargo doesn't have the git version, so we use "cargorel" as a
        // fallback value
        git_version!(fallback = &option_env!("NICKEL_NIX_BUILD_REV").unwrap_or("cargorel")[0..7])
    )
)]
struct Options {
    /// The trace output file, disables tracing if not given
    #[arg(short, long)]
    trace: Option<PathBuf>,

    /// If set, this process runs a background evaluation job instead of setting up a language server.
    #[arg(long)]
    background_eval: bool,
}

fn main() -> Result<()> {
    let handle = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    handle.join().unwrap()
}

fn run() -> Result<()> {
    use clap::Parser;

    env_logger::init();

    let options = Options::parse();

    if options.background_eval {
        return background::worker_main();
    }

    if let Some(file) = options.trace {
        let absolute_path = path::absolute(&file)?;
        debug!("Writing trace to {:?}", absolute_path);
        Trace::set_writer(csv::Writer::from_writer(io::BufWriter::new(
            fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(file)
                .map_err(|e| {
                    anyhow::anyhow!("Failed to open trace file {:?}: {}", absolute_path, e)
                })?,
        )))?;
    }

    let (connection, _threads) = Connection::stdio();

    let capabilities = Server::capabilities();

    let initialize_params = connection.initialize(serde_json::to_value(capabilities)?)?;

    debug!("Raw InitializeParams: {:?}", initialize_params);
    let config = match initialize_params.get("initializationOptions") {
        Some(opts) => serde_json::from_value::<LspConfig>(opts.clone())?,
        None => LspConfig::default(),
    };

    debug!("Parsed InitializeParams: {:?}", config);

    let _server = Server::new(connection, config).run();

    Ok(())
}
