## Background

It is difficult to test OpenSearch "painless" scripts in clusters without manual effort. These scripts can contain critical business logic and not being able to test them makes it difficult to have confidence deploying changes.

The goals of this project are to make it easy to run tests against OpenSearch painless scripts without affecting your production clusters.

Goals:

1. able to run tests locally as part of script development
2. configurable: set your OpenSearch version, set of scripts, cluster configuration, etc

Reference:

1. https://opensearch.org/docs/latest/api-reference/script-apis/exec-script/
2. https://www.elastic.co/guide/en/elasticsearch/painless/current/painless-lang-spec.html
3. Inspiration for other CLIs based on managing ElasticSearch clusters: https://github.com/LGUG2Z/elasdx

**Libraries**:

1. https://github.com/fussybeaver/bollard
2. https://docs.rs/clap/latest/clap/_derive/_cookbook/git_derive/index.html
3. https://opensearch.org/docs/latest/clients/rust/

**Why?**

To automate validation of painless scripts. See SO question(s) related to this.

1. https://stackoverflow.com/questions/48088139/how-to-validate-elasticsearch-painless-scripts
1. https://stackoverflow.com/questions/57362465/testing-functionality-of-painless-and-migration

To catch errors like this before script hits prod:

1. https://stackoverflow.com/questions/41348247/elasticsearch-painless-script-error

## Development Environment

You will need a test instance of OpenSearch running.

The recommended approach from the [docs](https://opensearch.org/docs/latest/install-and-configure/install-opensearch/docker/#run-opensearch-in-a-docker-container) is to run a single node cluster as follows.

```bash
# This command maps ports 9200 and 9600, sets the discovery type to "single-node" and requests the newest image of OpenSearch
docker run -d -p 9200:9200 -p 9600:9600 -e "discovery.type=single-node" public.ecr.aws/opensearchproject/opensearch:latest
```

Using [`finch`](https://runfinch.com):

```
# if your vm isn't already initalized
finch vm init
finch vm start

finch run -d -p 9200:9200 -p 9600:9600 -e "discovery.type=single-node" public.ecr.aws/opensearchproject/opensearch:latest
```

You should see a response like this:
```bash
curl https://localhost:9200 -ku 'admin:admin'
{
  "name" : "443b86add246",
  "cluster_name" : "docker-cluster",
  "cluster_uuid" : "ebctFi4SRWeHQ2n3Svs0qA",
  "version" : {
    "distribution" : "opensearch",
    "number" : "2.11.1",
    "build_type" : "tar",
    "build_hash" : "6b1986e964d440be9137eba1413015c31c5a7752",
    "build_date" : "2023-11-29T21:43:10.135035992Z",
    "build_snapshot" : false,
    "lucene_version" : "9.7.0",
    "minimum_wire_compatibility_version" : "7.10.0",
    "minimum_index_compatibility_version" : "7.0.0"
  },
  "tagline" : "The OpenSearch Project: https://opensearch.org/"
}
```

```bash
# example test
cargo run -- init --mapping ./mapping.json --index-name idx1 --username admin --password admin
```
