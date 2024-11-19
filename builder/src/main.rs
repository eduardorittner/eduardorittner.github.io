use comrak::{markdown_to_html, Options};
use gumdrop::Options as ArgOptions;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn format_header(title: &str, root: &str) -> String {
    format!(
        "<html>\
    <head> \
    <title>{}</title> \
    <link href=\"{}style.css\" rel=\"stylesheet\" type=\"text/css\" media=\"all\" \
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

enum ArticleKind {
    Home,
    Post,
    Note,
    Rambling,
}

fn format_navbar(prefix: &str, kind: ArticleKind) -> String {
    let mut home = "";
    let mut post = "";
    let mut note = "";

    match kind {
        ArticleKind::Home => home = "active",
        ArticleKind::Post => post = "active",
        ArticleKind::Note => note = "active",
        _ => (),
    }

    format!(
        "<body>\
        <div class=\"navbar\">\
        <a href=\"{prefix}index.html\" class=\"{home}\">Home</a>\
        <a href=\"{prefix}posts.html\" class=\"{post}\">Posts</a>\
        <a href=\"{prefix}notes.html\" class=\"{note}\">Notes</a>\
        </div><hr>
        "
    )
}

fn format_footer() -> String {
    "</body></html>".to_string()
}

#[derive(ArgOptions, Debug)]
struct Args {
    #[options(help = "root directory of .md files")]
    from: PathBuf,
    #[options(help = "root directory of generated site")]
    to: PathBuf,
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

#[derive(Debug)]
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

fn parse_header(contents: &str) -> (&str, Metadata) {
    let mut metadata = Metadata::default();
    let header_start = contents.strip_prefix("+++\n");

    let header_start = match header_start {
        None => return (contents, metadata),
        Some(a) => a,
    };

    let title_start = header_start
        .strip_prefix("title = ")
        .expect("Expected 'title = ' after header delimiter");

    let (title, rest) = title_start
        .split_once('\n')
        .expect("Expected '\n' after title parameter");

    metadata.title = title.to_string();

    let date_start = rest
        .strip_prefix("date = ")
        .expect("Expected 'date = ' after title declaration");

    let (date, rest) = date_start
        .split_once('\n')
        .expect("Expected '\n' after date parameter");

    metadata.date = Some(date.to_string());

    let (draft, rest) = if rest.starts_with("draft") {
        rest.split_once('\n')
            .expect("Expected line break after 'draft' parameter")
    } else {
        ("", rest)
    };

    let rest = rest
        .strip_prefix("+++\n")
        .expect("Expected '+++' delimiter");

    (rest, metadata)
}

fn md_file(path: &Path, root: &Path, to: PathBuf) {
    // TODO add header, navbar and footer
    let contents = std::fs::read_to_string(path).unwrap_or_default();

    let (contents, metadata) = parse_header(&contents);

    // Convert from .md to .html
    let html_content = markdown_to_html(&contents, &Options::default());

    let dest_string = to.to_str().unwrap_or_default();
    let path_string = path.to_str().unwrap_or_default();
    let root_string = root.to_str().unwrap_or_default();

    let (_, relative) = dest_string.split_once(root_string).unwrap_or_default();

    let depth = relative.chars().filter(|c| *c == '/').count() - 1;
    let prefix = if depth == 0 { "" } else { "../" };
    let html_header = format_header(&metadata.title, prefix);

    let html_navbar = if path_string.contains("posts/") {
        format_navbar(prefix, ArticleKind::Post)
    } else if path_string.contains("notes/") {
        format_navbar(prefix, ArticleKind::Note)
    } else if path_string.contains("ramblings/") {
        format_navbar(prefix, ArticleKind::Rambling)
    } else {
        format_navbar(prefix, ArticleKind::Home)
    };

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
    let opts = Args::parse_args_default_or_exit();

    // Create "to" dir if doesn't exist
    if !opts.to.exists() {
        std::fs::create_dir(opts.to.clone()).expect("Couldn't create dir: {opts.to:?}");
    }

    // Iterate recursively from "from" dir
    for entry in WalkDir::new(opts.from.clone())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = relative_path(entry.path(), &opts.from, &opts.to);

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
            md_file(entry.path(), &opts.to, path);
        } else {
            static_file(entry.path(), path);
        }
    }
}
