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
