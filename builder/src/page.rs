use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageKind {
    Index,
    Article,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Home,
    Post,
    Note,
    Rambling,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub title: String,
    pub date: Option<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            title: "Homepage".to_string(),
            date: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    pub content: String,
    pub kind: PageKind,
    pub category: Category,
    pub metadata: Metadata,
    pub path: PathBuf, // path relative to root
}

impl Page {
    pub fn new(path: &Path, link: &Path) -> Self {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Couldn't read file: {:?}", path));

        let kind = if path.ends_with("index.md")
            || path.ends_with("posts.md")
            || path.ends_with("notes.md")
            || path.ends_with("ramblings.md")
        {
            PageKind::Index
        } else {
            PageKind::Article
        };

        let category = if let Some(path) = path.parent() {
            if path.ends_with("posts/") {
                Category::Post
            } else if path.ends_with("notes/") {
                Category::Note
            } else if path.ends_with("ramblings/") {
                Category::Rambling
            } else if path.ends_with("src/") {
                Category::Home
            } else {
                unreachable!()
            }
        } else {
            Category::Home
        };

        let metadata = match kind {
            PageKind::Article => parse_header(&content),
            PageKind::Index => {
                let title = match category {
                    Category::Home => "Homepage",
                    Category::Post => "Posts",
                    Category::Note => "Notes",
                    Category::Rambling => "Ramblings",
                };
                Metadata {
                    title: title.to_string(),
                    date: None,
                }
            }
        };

        Self {
            kind,
            category,
            content,
            metadata,
            path: link.to_path_buf(),
        }
    }

    pub fn link(&self, root: &Path) -> String {
        let link = "https://eduardorittner.github.io/".to_owned();

        let addon = self
            .path
            .strip_prefix(root)
            .expect(&format!(
                "Couldn't find root: {root:?} in file: {:?}",
                self.path
            ))
            .to_string_lossy()
            .replace(".md", ".html");

        link + &addon
    }
}

fn parse_header(contents: &str) -> Metadata {
    let mut metadata = Metadata::default();
    let header_start = contents.strip_prefix("+++\n");

    let header_start = match header_start {
        None => return metadata,
        Some(a) => a,
    };

    let title_start = header_start
        .strip_prefix("title = ")
        .expect("Expected 'title = ' after header delimiter");

    let (title, rest) = title_start
        .split_once('\n')
        .expect("Expected '\n' after title parameter");

    let title = strip_string_delim(title);
    metadata.title = title.to_string();

    let date_start = rest
        .strip_prefix("date = ")
        .expect("Expected 'date = ' after title declaration");

    let (date, _) = date_start
        .split_once('\n')
        .expect("Expected '\n' after date parameter");

    if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date) {
        metadata.date = Some(date.to_rfc2822());
    } else {
        println!("invalid date: {date}")
    }

    metadata
}

fn strip_string_delim(s: &str) -> &str {
    if let Some(s) = s.strip_prefix("'") {
        s.strip_suffix("'").unwrap()
    } else {
        s.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap()
    }
}
