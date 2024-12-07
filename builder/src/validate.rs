use std::path::Path;
pub fn validate(path: &Path) {
    headers(path);
    use links::relative_links;
    relative_links(path);

    use links::url_links;
    url_links(path);
}

fn headers(to: &Path) {
    // TODO collect all errors before panicing, so it's easier to bulk fix
    use jwalk::WalkDir;
    WalkDir::new(to)
        .into_iter()
        .filter_map(|e| e.ok())
        .for_each(|entry| {
            if entry.file_type().is_file()
                && entry
                    .file_name()
                    .to_str()
                    .unwrap_or_default()
                    .ends_with(".html")
            {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let h1_number = content.matches("<h1").count();

                    if h1_number > 1 {
                        panic!("{} h1 headers in {:?}", h1_number, entry.path());
                    }
                }
            }
        });
}

mod links {
    use crate::builder::build;
    use jwalk::WalkDir;
    use rayon::prelude::*;
    use reqwest;
    use std::collections::VecDeque;
    use std::fs::read_to_string;
    use std::path::Path;
    use std::sync::Arc;

    struct LinkIterator<'a> {
        links: VecDeque<&'a str>,
    }

    impl<'a> LinkIterator<'a> {
        fn new_relative(s: &'a str) -> Self {
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

        fn new_url(s: &'a str) -> Self {
            let href = "href=\"";
            let mut links = VecDeque::new();
            let mut s = s;

            while let Some(start) = s.find(href) {
                s = &s[start + href.len()..];
                if let Some(end) = s.find("\"") {
                    let link = &s[..end];

                    if link.contains("http") {
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

    pub fn relative_links(to: &Path) {
        WalkDir::new(to)
            .into_iter()
            .filter_map(|e| e.ok())
            .for_each(|entry| {
                if entry.file_type().is_file()
                    && entry
                        .file_name()
                        .to_str()
                        .unwrap_or_default()
                        .ends_with(".html")
                {
                    if let Ok(content) = read_to_string(entry.path()) {
                        let iterator = LinkIterator::new_relative(&content);

                        iterator.links.par_iter().for_each(|link| {
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
                        });
                    };
                }
            });
    }

    pub fn url_links(to: &Path) {
        WalkDir::new(to)
            .into_iter()
            .filter_map(|e| e.ok())
            .for_each(|entry| {
                if entry.file_type().is_file()
                    && entry
                        .file_name()
                        .to_str()
                        .unwrap_or_default()
                        .ends_with(".html")
                {
                    if let Ok(content) = read_to_string(entry.path()) {
                        let iterator = LinkIterator::new_url(&content);

                        iterator.links.par_iter().for_each(|link| {
                            let full_path = entry.path();

                            if let Err(e) = reqwest::blocking::get(*link) {
                                panic!("{e} from {full_path:?}");
                            };
                        });
                    }
                }
            });
    }
}
