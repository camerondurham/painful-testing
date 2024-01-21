use clap::Parser;
use opensearch::cert::CertificateValidation;
use opensearch::http::Url;
use opensearch::OpenSearch;
use painful_testing::cli::{Cli, Commands};
use serde_json::Value;
use std::fs;

use opensearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use painful_testing::opensearch_util::get_local_client;
use painful_testing::painless::{DocRef, TestCase};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let args = Cli::parse();

    match args.command {
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
            let client = get_local_client()?;

            // let url = Url::parse("https://localhost:9200")?;
            // let conn_pool = SingleNodeConnectionPool::new(url.clone());
            // let transport = TransportBuilder::new(conn_pool)
            //     .proxy(url, Some("admin"), Some("admin"))
            //     .build()?;
            // let client = OpenSearch::new(transport);
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
            let conn_pool = SingleNodeConnectionPool::new(url);

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
