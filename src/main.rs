//! cfdns - Cloudflare DNS record manager CLI.

mod api;
mod cli;
mod commands;
mod config;
mod models;
mod output;

use api::CloudflareClient;
use cli::{Cli, Commands};
use commands::*;
use config::Config;
use models::Result;
use output::OutputFormat;

use clap::Parser;
use std::process;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let fmt = cli.output;

    match cli.command {
        // Init doesn't need config/client
        Commands::Init(args) => cmd_init(args),

        // All other commands need config + client
        other => {
            let config = Config::load(cli.profile.as_deref())?;
            let client = CloudflareClient::new(config);
            dispatch(&client, other, fmt)
        }
    }
}

fn dispatch(client: &CloudflareClient, cmd: Commands, fmt: OutputFormat) -> Result<()> {
    match cmd {
        Commands::List(args) => cmd_list(client, args, fmt),
        Commands::Get(args) => cmd_get(client, args, fmt),
        Commands::Create(args) => cmd_create(client, args, fmt),
        Commands::Update(args) => cmd_update(client, args, fmt),
        Commands::Overwrite(args) => cmd_overwrite(client, args, fmt),
        Commands::Delete(args) => cmd_delete(client, args, fmt),
        Commands::Search(args) => cmd_search(client, args, fmt),
        Commands::Count(args) => cmd_count(client, args),
        Commands::Comment(args) => cmd_comment(client, args, fmt),
        Commands::Tag(args) => cmd_tag(client, args, fmt),
        Commands::Init(_) => unreachable!(),
    }
}
