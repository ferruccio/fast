use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sts::Client;
use clap::Parser;
use ini::Ini;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Options {
    #[clap(index = 1, required = true, help = "AWS credentials file")]
    credentials: PathBuf,

    #[clap(index = 2, required = true, help = "AWS profile to update")]
    profile: String,
}

#[tokio::main]
async fn main() {
    let opts = Options::parse();

    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    match client.get_session_token().send().await {
        Ok(response) => {
            if let Some(credentials) = response.credentials {
                let mut conf = Ini::load_from_file(&opts.credentials)
                    .expect("failed to open credentials file");
                conf.set_to(
                    Some(&opts.profile),
                    String::from("aws_access_key_id"),
                    credentials
                        .access_key_id
                        .unwrap_or(String::from("no aws access key id")),
                );
                conf.set_to(
                    Some(&opts.profile),
                    String::from("aws_secret_access_key"),
                    credentials
                        .secret_access_key
                        .unwrap_or(String::from("no aws secret access key")),
                );
                conf.set_to(
                    Some(&opts.profile),
                    String::from("aws_session_token"),
                    credentials
                        .session_token
                        .unwrap_or(String::from("no aws session token")),
                );
                conf.write_to_file(&opts.credentials)
                    .expect("failed to update credentials file");
            }
        }
        Err(err) => println!("{err}"),
    }
}
