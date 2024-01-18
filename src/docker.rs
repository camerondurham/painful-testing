use std::collections::HashMap;

use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::Docker;

pub async fn start_os_container(version: &str) -> Result<(), Box<dyn std::error::Error + 'static>> {
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
