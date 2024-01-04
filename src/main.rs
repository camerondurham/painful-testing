use bollard::container::{Config, CreateContainerOptions};
use bollard::Docker;
use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
enum Commands {
    /// starts local OpenSearch instance
    #[command(arg_required_else_help = false)]
    Start {},
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
        Commands::Start {} => {
            run_docker().await.expect("Unable to run Docker");
        }
    }

    Ok(())
}

async fn run_docker() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let opensearch_config = Config {
        // TODO: accept version / custom image name from command or config
        image: Some("public.ecr.aws/opensearchproject/opensearch:1.3.13"),
        env: Some(vec!["TEST=1234"]),
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
    Ok(())
}
