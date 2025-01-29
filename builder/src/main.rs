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
        if let Err(e) = site.build().await {
            println!("ERROR: {:?}", e);
            // Exit with an error
            std::process::exit(1);
        };
    } else {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        let validator = ExternalLinkValidator(rx);
        // TODO handle error
        let validator = tokio::spawn(validator.run_validator());

        {
            let site = Arc::new(Site::new(to, root, Some(tx)));

            if let Err(e) = site.clone().build().await {
                println!("ERROR: {e:?}");
                // Exit with an error
                std::process::exit(1);
            }
        }

        println!("Checking url links");
        // Wait for external url validator to finish before exiting
        if let Ok(Err(e)) = validator.await {
            println!("ERROR: {e:?}");
        }
    }
}
