use crate::*;
use comrak::{
    adapters, markdown_to_html_with_plugins, plugins, Options, PluginsBuilder, RenderPluginsBuilder,
};
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
        write!(output, "<h{}>", level)
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
                "</h{}>\
                <div class=\"small-break\"></div>\
                </div>",
                level,
            )
        } else {
            write!(output, "</h{}>", level)
        }
    }
}

pub fn to_html(content: &str) -> String {
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

    markdown_to_html_with_plugins(content, &options, &plugin)
}

pub fn md_file(path: &Path, root: &Path, to: PathBuf) {
    let page = Page::new(path);

    let mut html_content = format_metadata(&page.metadata);

    let content = to_html(&page.content);

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
}

pub fn build(root: &Path, to: &Path) {
    if !to.exists() {
        std::fs::create_dir(to).unwrap();
    }

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
            md_file(entry.path(), to, path);
        } else {
            static_file(entry.path(), path);
        }
    }
}
