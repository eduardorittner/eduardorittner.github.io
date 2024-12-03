use builder::builder::build;
use std::collections::VecDeque;
use std::fs::read_to_string;
use std::path::Path;
use walkdir::WalkDir;

struct LinkIterator<'a> {
    links: VecDeque<&'a str>,
}

impl<'a> LinkIterator<'a> {
    fn new(s: &'a str) -> Self {
        let href = "href=\"";
        let mut links = VecDeque::new();
        let mut s = s;

        while let Some(start) = s.find(href) {
            s = &s[start + href.len()..];
            if let Some(end) = s.find("\"") {
                let link = &s[..end];

                if !link.contains("http") {
                    links.push_front(link);
                }
            }
        }

        Self { links }
    }
}

impl<'a> Iterator for LinkIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.links.pop_front()
    }
}

#[test]
fn broken_links() {
    let root = Path::new("../src");
    let to = Path::new("/tmp/output");
    build(root, to);

    for entry in WalkDir::new(to).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && entry
                .file_name()
                .to_str()
                .unwrap_or_default()
                .ends_with(".html")
        {
            if let Ok(content) = read_to_string(entry.path()) {
                let iterator = LinkIterator::new(&content);

                for link in iterator {
                    let full_path = entry.path().parent().unwrap().join(&Path::new(&link));

                    if !full_path.exists() {
                        panic!(
                            "{}",
                            format!(
                                "path {:?} from {:?} does not exist",
                                full_path.strip_prefix(to).unwrap(),
                                entry.path(),
                            )
                        );
                    }
                }
            };
        }
    }
}
