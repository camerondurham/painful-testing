use std::collections::HashMap;
use std::ffi::OsString;

use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::Docker;
use clap::{Parser, Subcommand};
use painful_testing::{DocRef, TestCase};

#[derive(Subcommand, Debug)]
enum Commands {
    /// starts local OpenSearch instance
    #[command(arg_required_else_help = false)]
    Start {
        #[arg(default_missing_value = "latest", default_value = "1.3.13")]
        version: Option<OsString>,
    },
    /// runs single test case on OpenSearch provided instance
    #[command(arg_required_else_help = false)]
    Test {
        doc_id: OsString,
        current_state: OsString,
        incoming: OsString,
        expected: OsString,
    },
}

#[derive(Debug, Parser)]
#[command(name = "pf")]
#[command(about = "A CLI to run painless script tests on OpenSearch clusters", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let args = Cli::parse();

    match args.command {
        Commands::Start { version } => {
            start_os_container(
                version
                    .expect("unable to find version arg")
                    .as_os_str()
                    .to_str()
                    .unwrap(),
            )
            .await
            .expect("Unable to run Docker");
        }
        Commands::Test {
            doc_id,
            current_state,
            incoming,
            expected,
        } => {
            let test_case = TestCase {
                id: doc_id.to_str().unwrap(),
                state: Some(DocRef::Filepath(
                    current_state.to_str().unwrap().to_string(),
                )),
                incoming: DocRef::Filepath(incoming.to_str().unwrap().to_string()),
                expected: Some(DocRef::Filepath(expected.to_str().unwrap().to_string())),
            };

            println!("Running test case: {:?}", test_case);
        }
    }

    Ok(())
}

async fn start_os_container(version: &str) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let mut hm = HashMap::<&str, HashMap<(), ()>>::new();
    let inner_map = HashMap::<(), ()>::new();
    hm.insert("9100:9100", inner_map);
    let image = format!("public.ecr.aws/opensearchproject/opensearch:{}", version);

    let opensearch_config = Config {
        // TODO: accept version / custom image name from command or config
        image: Some(image.as_str()),
        env: Some(vec!["discovery.type=single-node"]),
        exposed_ports: Some(hm),
        ..Default::default()
    };

    let _ = &docker
        .create_container(
            Some(CreateContainerOptions {
                name: "opensearch",
                platform: None,
            }),
            opensearch_config,
        )
        .await?;
    let _ = docker
        .start_container("opensearch", None::<StartContainerOptions<String>>)
        .await?;

    Ok(())
}
