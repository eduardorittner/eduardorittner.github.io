use crate::*;
use ::rss::Item;
use comrak::{
    adapters, markdown_to_html_with_plugins, plugins, Options, PluginsBuilder, RenderPluginsBuilder,
};
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

struct Heading;

impl adapters::HeadingAdapter for Heading {
    fn enter(
        &self,
        output: &mut dyn std::io::Write,
        heading: &adapters::HeadingMeta,
        _sourcepos: Option<comrak::nodes::Sourcepos>,
    ) -> std::io::Result<()> {
        let level = if heading.level < 6 {
            heading.level + 1
        } else {
            6
        };

        if level < 3 {
            write!(
                output,
                "<div class=\"heading-container\">\
                <div class=\"small-break\"></div>"
            )
            .unwrap();
        }

        write!(
            output,
            "<h{} id=\"{}\">",
            level,
            heading.content.replace(" ", "-")
        )
    }

    fn exit(
        &self,
        output: &mut dyn std::io::Write,
        heading: &adapters::HeadingMeta,
    ) -> std::io::Result<()> {
        let level = if heading.level < 6 {
            heading.level + 1
        } else {
            6
        };

        if level < 3 {
            write!(
                output,
                "<a class=\"header-link\" href=\"#{}\">\
                <img src=\"../assets/link.svg\" class=\"link-icon\" alt=\"Link icon\">\
                </a>\
                </h{}>\
                <div class=\"small-break\"></div>\
                </div>",
                heading.content.replace(" ", "-"),
                level,
            )
        } else {
            write!(output, "</h{}>", level)
        }
    }
}

pub fn to_html(page: &Page) -> String {
    let mut options = Options::default();
    options.extension.front_matter_delimiter = Some("+++".to_owned());

    let heading_adapter = Heading;
    let syntax_adapter = plugins::syntect::SyntectAdapter::new(Some("base16-mocha.dark"));
    let render_plugin = RenderPluginsBuilder::default()
        .heading_adapter(Some(&heading_adapter))
        .codefence_syntax_highlighter(Some(&syntax_adapter))
        .build()
        .unwrap();

    let plugin = PluginsBuilder::default()
        .render(render_plugin)
        .build()
        .unwrap();

    let html = markdown_to_html_with_plugins(&page.content, &options, &plugin);

    match (page.category, page.kind) {
        (Category::Post, PageKind::Article) => table_of_contents(html),
        (_, _) => html,
    }
}

pub fn md_file(path: &Path, root: &Path, to: PathBuf) -> Item {
    let page = Page::new(path, to.strip_prefix(root).unwrap());

    let mut html_content = format_metadata(&page.metadata);

    let content = to_html(&page);

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

    // Write to file
    std::fs::write(to.clone(), html).unwrap_or_else(|_| panic!("Couldn't write to file: {:?}", to));
    new_item(&page)
}

pub fn build(root: &Path, to: &Path) {
    if !to.exists() {
        std::fs::create_dir(to).unwrap();
    }

    let mut rss_items = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = relative_path(entry.path(), root, to);

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
            rss_items.push(md_file(entry.path(), to, path));
        } else {
            static_file(entry.path(), path);
        }
    }

    let rss_feed = new_rss(rss_items);

    let mut rss_path = to.to_path_buf();
    rss_path.push("rss.xml");
    let file = File::create(&rss_path).unwrap();
    rss_feed.pretty_write_to(&file, b' ', 2).unwrap();
}
