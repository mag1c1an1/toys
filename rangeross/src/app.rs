use std::env;

use color_eyre::Result;
use opendal::services::S3;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    endpoint: String,
    id: String,
    key: String,
    bucket: String,
    region: String,
}

pub fn s3_try_from_config() -> Result<S3> {
    let yaml_str = std::fs::read_to_string("config.yaml")?;
    let config: Config = serde_yaml::from_str(&yaml_str)?;
    println!("{:?}", config);
    Ok(S3::default()
        .endpoint(&config.endpoint)
        .access_key_id(&config.id)
        .secret_access_key(&config.key)
        .bucket(&config.bucket)
        .region(&config.region))
}

pub fn s3_try_from_env() -> Result<S3> {
    let endpoint = env::var("AWS_ENDPOINT")?;
    let id = env::var("AWS_ACCESS_KEY_ID")?;
    let key = env::var("AWS_SECRET_ACCESS_KEY")?;
    let bucket = env::var("AWS_BUCKET")?;
    let region = env::var("AWS_REGION").unwrap_or(String::from("auto"));
    println!("{} {} {} {} {} ", endpoint, id, key, bucket, region);
    Ok(S3::default()
        .endpoint(&endpoint)
        .access_key_id(&id)
        .secret_access_key(&key)
        .bucket(&bucket)
        .region(&region)
    )
}

mod tests {
    use super::*;
    use opendal::{Operator, layers::LoggingLayer};
    use test_log::test;

    #[test(tokio::test)]
    async fn s3_test() {
        let op = Operator::new(s3_try_from_env().unwrap())
            .unwrap()
            .layer(LoggingLayer::default())
            .finish();
        let a = op.list("").await.unwrap();
        for e in a {
            println!("{}", e.name());
        }
    }
}
