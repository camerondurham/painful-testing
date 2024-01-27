use clap::Parser;
use opensearch::cert::CertificateValidation;
use opensearch::http::Url;
use opensearch::OpenSearch;
use painful_testing::cli::{Cli, Commands};
use serde_json::Value;
use std::fs;

use opensearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use painful_testing::opensearch_util::{get_client, get_local_client};
use painful_testing::painless::{DocRef, TestCase};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let args = Cli::parse();

    match args.command {
        Commands::CreateIndex {
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

            if response.status_code().is_success() {
                println!("Successfully created index");
            } else {
                let resp = response.json::<Value>().await?;
                println!("Failed to create index: {:?}", resp);
            }
        }
        Commands::PutScript {
            cluster_url,
            username,
            password,
            script_path,
            script_id,
        } => {
            let contents = fs::read_to_string(script_path.to_str().unwrap())
                .expect("Should have read file to string");

            let url = Url::parse(cluster_url.to_str().unwrap())?;
            let conn_pool = SingleNodeConnectionPool::new(url);

            let transport = TransportBuilder::new(conn_pool)
                .auth(opensearch::auth::Credentials::Basic(
                    username.to_str().unwrap().to_string(),
                    password.to_str().unwrap().to_string(),
                ))
                .cert_validation(CertificateValidation::None)
                .build()?;

            let client = OpenSearch::new(transport);
            let response = client
                .put_script(opensearch::PutScriptParts::Id(script_id.to_str().unwrap()))
                .body(contents)
                .send()
                .await?;
            if response.status_code().is_success() {
                println!("Successfully wrote script to cluster");
            } else {
                let resp = response.json::<Value>().await?;
                println!("Failed to create index: {:?}", resp);
            }
        }
        Commands::IndexDocument {
            cluster_url,
            username,
            password,
            index_name,
            doc_path,
        } => {
            println!("writing doc {:?} to {:?} index", doc_path, index_name);
            let client = get_client(
                cluster_url.to_str().unwrap(),
                username.to_str().unwrap(),
                password.to_str().unwrap(),
            )
            .expect("unable to create OpenSearch client");
        }
    }

    Ok(())
}
