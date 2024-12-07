use builder::builder::build;
use std::path::Path;

fn main() {
    let root = Path::new("../src");
    let to = Path::new("../output");
    build(root, to);

    #[cfg(feature = "validate")]
    builder::validate::validate(to);
}
