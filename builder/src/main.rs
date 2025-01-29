use builder::builder::{ExternalLinkValidator, Site};
use std::{path::PathBuf, sync::Arc};
use tokio;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let root = PathBuf::from("../src");
    let to = PathBuf::from("../output");

    if args.get(1).is_none_or(|s| s != "validate") {
        if let Err(e) = Site::build(to, root, None).await {
            println!("ERROR: {:?}", e);
            std::process::exit(1);
        };
    } else {
        if let Err(e) = Site::build_with_url_validator(to, root).await {
            println!("ERROR: {:?}", e);
            std::process::exit(1);
        }
    }
}
