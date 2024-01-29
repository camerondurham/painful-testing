use clap::{Parser, Subcommand};
use opensearch::cert::CertificateValidation;
use opensearch::http::Url;
use opensearch::OpenSearch;
use serde_json::Value;
use std::fs;

use opensearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use painful_testing::opensearch_util::get_client;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// initalize cluster with mapping configuration
    #[command(arg_required_else_help = false)]
    CreateIndex {
        // TODO: find how to set these as defaults
        #[arg(
            short,
            long,
            default_value = "https://localhost:9200",
            default_missing_value = "https://localhost:9200"
        )]
        cluster_url: String,
        #[arg(short, long, default_value = "admin", default_missing_value = "admin")]
        username: String,
        #[arg(short, long, default_value = "admin", default_missing_value = "admin")]
        password: String,

        #[arg(short, long)]
        mapping: String,
        #[arg(short, long)]
        index_name: String,
    },
    #[command(arg_required_else_help = false)]
    PutScript {
        // TODO: find how to set these as defaults
        #[arg(
            short,
            long,
            default_value = "https://localhost:9200",
            default_missing_value = "https://localhost:9200"
        )]
        cluster_url: String,
        #[arg(short, long, default_value = "admin", default_missing_value = "admin")]
        username: String,
        #[arg(short, long, default_value = "admin", default_missing_value = "admin")]
        password: String,

        #[arg(short, long)]
        script_path: String,
        #[arg(short, long)]
        script_id: String,
    },
    #[command(arg_required_else_help = false)]
    IndexDocument {
        // TODO: find how to set these as defaults
        #[arg(
            short,
            long,
            default_value = "https://localhost:9200",
            default_missing_value = "https://localhost:9200"
        )]
        cluster_url: String,
        #[arg(short, long, default_value = "admin", default_missing_value = "admin")]
        username: String,
        #[arg(short, long, default_value = "admin", default_missing_value = "admin")]
        password: String,
        #[arg(short, long)]
        index_name: String,
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        doc_path: String,
    },
}

#[derive(Debug, Parser)]
#[command(name = "pf")]
#[command(about = "A CLI to run painless script tests on OpenSearch clusters", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

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

            let url = Url::parse(cluster_url.as_str())?;
            let conn_pool = SingleNodeConnectionPool::new(url);

            // for local testing, ignore certificate validation
            let transport = TransportBuilder::new(conn_pool)
                .auth(opensearch::auth::Credentials::Basic(username, password))
                .cert_validation(CertificateValidation::None)
                .build()?;

            let client = OpenSearch::new(transport);
            let response = client
                .indices()
                .get_mapping(opensearch::indices::IndicesGetMappingParts::Index(&[
                    index_name.as_str(),
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

            let mapping_content = fs::read_to_string(mapping.as_str())?;
            println!("mapping content: {:?}", &mapping_content);
            let json_mapping_content: serde_json::Value =
                serde_json::from_str(&mapping_content.as_str())?;
            println!("json mapping content: {:?}", &json_mapping_content);

            let response = client
                .indices()
                .create(opensearch::indices::IndicesCreateParts::Index(
                    index_name.as_str(),
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
            let contents =
                fs::read_to_string(script_path).expect("Should have read file to string");

            let url = Url::parse(&cluster_url)?;
            let conn_pool = SingleNodeConnectionPool::new(url);

            let transport = TransportBuilder::new(conn_pool)
                .auth(opensearch::auth::Credentials::Basic(
                    username.to_string(),
                    password.to_string(),
                ))
                .cert_validation(CertificateValidation::None)
                .build()?;

            let client = OpenSearch::new(transport);
            let response = client
                .put_script(opensearch::PutScriptParts::Id(&script_id))
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
            id,
        } => {
            println!("writing doc {:?} to {:?} index", doc_path, index_name);
            let client = get_client(&cluster_url, &username, &password)
                .expect("unable to create OpenSearch client");
            let doc_body = fs::read_to_string(doc_path)?;
            client
                .index(opensearch::IndexParts::IndexId(
                    index_name.as_str(),
                    id.as_str(),
                ))
                .body(doc_body)
                .send()
                .await?;
        }
    }

    Ok(())
}
