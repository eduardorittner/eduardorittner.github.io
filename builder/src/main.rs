use builder::builder::{ExternalLinkValidator, Site};
use std::{path::PathBuf, sync::Arc};
use tokio;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let root = PathBuf::from("../src");
    let to = PathBuf::from("../output");

    if args.get(1).is_none_or(|s| s != "validate") {
        let site = Arc::new(Site::new(to, root, None));
        // TODO handle error
        site.build().await.unwrap();
    } else {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        let validator = ExternalLinkValidator(rx);
        // TODO handle error
        let validator = tokio::spawn(validator.run_validator());

        {
            let site = Arc::new(Site::new(to, root, Some(tx)));

            // TODO handle errors
            site.clone().build().await.unwrap();
        }

        // Wait for external url validator to finish before exiting
        let b = validator.await.unwrap();
        println!("{b:?}");
    }
}
