use comrak::{markdown_to_html, Options};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn format_header(title: &str, root: &str) -> String {
    format!(
        "<html>\
    <head> \
    <title>{}</title> \
    <link href=\"{}style.css\" rel=\"stylesheet\" type=\"text/css\" media=\"all\" \
    <link href='https://fonts.googleapis.com/css?family=Fira Mono' rel='stylesheet'> \
    <meta charset=\"UTF-8\"> \
    <script type=\"text/x-mathjax-config\"> \
    MathJax.Hub.Config({{ \
    tex2jax: {{inlineMath: [['$','$'], ['\\\\(','\\\\)']]}} \
    }}); \
    </script> \
    <script type=\"text/javascript\" \
    src=\"http://cdn.mathjax.org/mathjax/latest/MathJax.js?config=TeX-AMS-MML_HTMLorMML\"> \
    </script> \
    </head> \
    ",
        title, root
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageKind {
    Index,
    Article,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    Home,
    Post,
    Note,
    Rambling,
}

#[derive(Debug, Clone)]
struct Page {
    content: String,
    kind: PageKind,
    category: Category,
    metadata: Metadata,
}

impl Page {
    fn new(path: &Path) -> Self {
        let content =
            std::fs::read_to_string(path).expect(&format!("Couldn't read file: {:?}", path));

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
        }
    }
}

fn format_navbar(prefix: &str, kind: Category) -> String {
    let mut home = "";
    let mut post = "";
    let mut note = "";

    match kind {
        Category::Home => home = "active",
        Category::Post => post = "active",
        Category::Note => note = "active",
        _ => (),
    }

    format!(
        "<body>\
        <div class=\"navbar\">\
        <a href=\"{prefix}index.html\" class=\"{home}\">Home</a>\
        <a href=\"{prefix}posts/posts.html\" class=\"{post}\">Posts</a>\
        <a href=\"{prefix}notes/notes.html\" class=\"{note}\">Notes</a>\
        </div>
        "
    )
}

fn format_footer() -> String {
    "</article></body></html>".to_string()
}

fn relative_path(path: &Path, from: &Path, to: &Path) -> PathBuf {
    let relative_path = path
        .to_str()
        .unwrap_or_default()
        .trim_start_matches(from.to_str().unwrap_or_default())
        .strip_prefix("/") // Strip '/' because of .join behavior on absolute paths
        .unwrap_or_default()
        .replace(".md", ".html")
        .to_string();
    to.join(Path::new(&relative_path))
}

fn static_file(path: &Path, to: PathBuf) {
    std::fs::copy(path, to).expect("Couldn't copy static file");
}

#[derive(Debug, Clone)]
struct Metadata {
    title: String,
    date: Option<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            title: "Homepage".to_string(),
            date: None,
        }
    }
}

fn strip_string_delim(s: &str) -> &str {
    if let Some(s) = s.strip_prefix("'") {
        s.strip_suffix("'").unwrap()
    } else {
        s.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap()
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

    metadata.date = Some(date.to_string());

    metadata
}

fn format_metadata(metadata: &Metadata) -> String {
    let mut title = format!(
        "<article id=\"post\"><div class=\"stack\"><h1>{}</h1>",
        metadata.title
    );
    if let Some((date, _)) = metadata.date.clone().unwrap_or_default().split_once('T') {
        let date = format!("<span class=\"date\">Published: {}</span>", date);
        title.push_str(&date);
    }
    title.push_str("</div>");
    title
}

fn md_file(path: &Path, root: &Path, to: PathBuf) {
    let page = Page::new(path);

    let mut html_content = format_metadata(&page.metadata);

    // Convert from .md to .html
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("+++".to_owned());
    let content = markdown_to_html(&page.content, &options);

    html_content.push_str(&content);

    let dest_string = to.to_str().unwrap_or_default();
    let root_string = root.to_str().unwrap_or_default();

    let (_, relative) = dest_string.split_once(root_string).unwrap_or_default();

    let depth = relative.chars().filter(|c| *c == '/').count() - 1;
    let prefix = if depth == 0 { "" } else { "../" };
    let html_header = format_header(&page.metadata.title, prefix);
    let html_navbar = format_navbar(prefix, page.category);
    let html_footer = format_footer();

    let html = html_header + &html_navbar + &html_content + &html_footer;

    // TODO downsize all headings one level, so the post title is h1 and
    // the '# title' blocks in markdown correspond to h2 and below
    // mayber using [heading adapter](https://docs.rs/comrak/latest/comrak/adapters/trait.HeadingAdapter.html)?
    // Could just copy their implementation and write nch.level - 1
    // [implementation](https://docs.rs/comrak/latest/comrak/adapters/trait.HeadingAdapter.html)

    // Write to file
    std::fs::write(to.clone(), html).expect(&format!("Couldn't write to file: {:?}", to));
}

fn main() {
    let root = Path::new("../src");
    let to = Path::new("../output");
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = relative_path(entry.path(), &root, &to);

        // Create child dir if doesn't exist
        if entry.file_type().is_dir() {
            if !path.exists() {
                std::fs::create_dir(path).unwrap();
            }
        } else if entry
            .file_name()
            .to_str()
            .unwrap_or_default()
            .ends_with(".md")
        {
            md_file(entry.path(), &to, path);
        } else {
            static_file(entry.path(), path);
        }
    }
}
