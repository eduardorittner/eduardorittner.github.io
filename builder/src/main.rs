use pandalib::builder::Site;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from("../src");
    let to = PathBuf::from("../output");

    let _ = Site::build(to, root);
}
