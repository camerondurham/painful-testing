use std::ffi::OsString;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// starts local OpenSearch instance
    #[command(arg_required_else_help = false)]
    Start {
        #[arg(default_missing_value = "latest", default_value = "1.3.13")]
        version: Option<OsString>,
    },
    /// runs single test case on OpenSearch provided instance
    #[command(arg_required_else_help = false)]
    Test {
        #[arg(short, long)]
        doc_id: OsString,
        #[arg(short, long)]
        current_state: OsString,
        #[arg(short, long)]
        incoming: OsString,
        #[arg(short, long)]
        expected: OsString,
    },
    /// initalize cluster with mapping configuration
    #[command(arg_required_else_help = false)]
    Init {
        // TODO: find how to set these as defaults
        #[arg(
            short,
            long,
            default_value = "https://localhost:9200",
            default_missing_value = "https://localhost:9200"
        )]
        cluster_url: OsString,
        #[arg(short, long, default_value = "admin", default_missing_value = "admin")]
        username: OsString,
        #[arg(short, long, default_value = "admin", default_missing_value = "admin")]
        password: OsString,

        #[arg(short, long)]
        mapping: OsString,
        #[arg(short, long)]
        index_name: OsString,
    },
}

#[derive(Debug, Parser)]
#[command(name = "pf")]
#[command(about = "A CLI to run painless script tests on OpenSearch clusters", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
