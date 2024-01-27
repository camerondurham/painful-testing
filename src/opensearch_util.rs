use opensearch::cert::CertificateValidation;
use opensearch::http::Url;
use opensearch::OpenSearch;

use opensearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};

pub fn get_local_client() -> Result<OpenSearch, Box<dyn std::error::Error + 'static>> {
    // TODO: implement customization of where the opensearch cluster is running
    // Currently this just defaults to using https://localhost:9200 -ku admin:admin
    let url = Url::parse("https://localhost:9200")?;
    let conn_pool = SingleNodeConnectionPool::new(url.clone());
    let transport = TransportBuilder::new(conn_pool)
        .auth(opensearch::auth::Credentials::Basic(
            "admin".to_string(),
            "admin".to_string(),
        ))
        .cert_validation(CertificateValidation::None)
        .build()?;
    let client = OpenSearch::new(transport);
    Ok(client)
}

pub fn get_client(
    cluster_url: &str,
    username: &str,
    password: &str,
) -> Result<OpenSearch, Box<dyn std::error::Error + 'static>> {
    let url = Url::parse(cluster_url)?;
    let conn_pool = SingleNodeConnectionPool::new(url);

    let transport = TransportBuilder::new(conn_pool)
        .auth(opensearch::auth::Credentials::Basic(
            username.to_string(),
            password.to_string(),
        ))
        .cert_validation(CertificateValidation::None)
        .build()?;

    Ok(OpenSearch::new(transport))
}
