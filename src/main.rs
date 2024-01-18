use opensearch::cert::CertificateValidation;
use opensearch::http::Url;
use opensearch::OpenSearch;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;

use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::Docker;
use clap::{Parser, Subcommand};
use opensearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
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

            // TODO: implement customization of where the opensearch cluster is running
            // Currently this just defaults to using https://localhost:9200 -ku admin:admin
            let url = Url::parse("https://localhost:9200")?;
            let conn_pool = SingleNodeConnectionPool::new(url.clone());
            let transport = TransportBuilder::new(conn_pool)
                .proxy(url, Some("admin"), Some("admin"))
                .build()?;
            let client = OpenSearch::new(transport);
            let nodes = client.nodes();
            let stats = nodes.stats(opensearch::nodes::NodesStatsParts::NodeId(&["_all"]));
            println!("{:?}", stats.pretty(true));
        }
        Commands::Init {
            mapping,
            index_name,
            cluster_url,
            username,
            password,
        } => {
            println!(
                "Initalizing cluster (url={:?}) with config: {:?} {:?}",
                cluster_url, mapping, index_name
            );

            let url = Url::parse(cluster_url.to_str().unwrap())?;
            let conn_pool = SingleNodeConnectionPool::new(url.clone());

            // for local testing, ignore certificate validation
            let transport = TransportBuilder::new(conn_pool)
                .auth(opensearch::auth::Credentials::Basic(
                    username.to_str().unwrap().to_string(),
                    password.to_str().unwrap().to_string(),
                ))
                .cert_validation(CertificateValidation::None)
                .build()?;

            let client = OpenSearch::new(transport);
            let response = client
                .indices()
                .get_mapping(opensearch::indices::IndicesGetMappingParts::Index(&[
                    index_name.to_str().unwrap(),
                ]))
                .send()
                .await?;
            println!("Response: {:?}", response);
            if response.status_code().is_success() {
                println!("Successfully received get mapping response: {:?}", response);
            } else {
                let resp = response.json::<Value>().await?;
                println!("Failed to get mapping response: {:?}", resp);
            }

            // TODO: put mapping
            // https://docs.rs/opensearch/latest/opensearch/indices/struct.Indices.html#method.put_mapping

            let mapping_content = fs::read_to_string(mapping.to_str().unwrap())?;
            println!("mapping content: {:?}", &mapping_content);
            let json_mapping_content: serde_json::Value =
                serde_json::from_str(&mapping_content.as_str())?;
            println!("json mapping content: {:?}", &json_mapping_content);

            // let response = client
            //     .indices()
            //     .put_mapping(opensearch::indices::IndicesPutMappingParts::Index(&[
            //         index_name.to_str().unwrap(),
            //     ]))
            //     .body(json_mapping_content.as_object().unwrap())
            //     .send()
            //     .await?;
            let response = client
                .indices()
                .create(opensearch::indices::IndicesCreateParts::Index(
                    index_name.to_str().unwrap(),
                ))
                .body(json_mapping_content.as_object().unwrap())
                .send()
                .await?;

            let successful = response.status_code().is_success();
            if successful {
                println!("Successfully created index");
            } else {
                let resp = response.json::<Value>().await?;
                println!("Failed to create index: {:?}", resp);
            }
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
