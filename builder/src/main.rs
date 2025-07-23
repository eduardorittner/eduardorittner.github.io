use pandalib::builder::Site;
use std::{path::PathBuf, sync::Arc};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let root = PathBuf::from("../src");
    let to = PathBuf::from("../output");

    Site::build(to, root);
}
