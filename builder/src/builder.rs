use crate::*;
use comrak::{
    adapters, markdown_to_html_with_plugins, plugins, Options, PluginsBuilder, RenderPluginsBuilder,
};
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use walkdir::WalkDir;

pub struct ExternalLinkValidator(pub Receiver<UrlLink>);

impl ExternalLinkValidator {
    pub async fn run_validator(mut self) -> Result<(), Vec<reqwest::Error>> {
        let mut errors: Vec<reqwest::Error> = Vec::new();

        while let Some(link) = self.0.recv().await {
            if let Err(e) = reqwest::get(link.0.link).await {
                // TODO wrap errors with context, which file the link is from
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone)]
struct Link {
    link: String,
    file: PathBuf,
}

struct RelativeLink(Link);

#[derive(Debug)]
pub struct UrlLink(Link);

pub enum BuildError {
    InvalidLink(Link),
    // TODO define error from tokio::fs operations
}

impl std::fmt::Debug for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::InvalidLink(link) => {
                write!(f, "Invalid link: {} from file: {:?}", link.link, link.file)
            }
        }
    }
}

struct GeneratedHtml {
    to: PathBuf,
    from: PathBuf,
    content: String,
}

struct AssetFile {
    to: PathBuf,
    from: PathBuf,
}

pub struct Site {
    dest: PathBuf, // Path to dest dir
    root: PathBuf, // Path to root dir
    assets: Arc<Mutex<Vec<AssetFile>>>,
    pages: Arc<Mutex<Vec<GeneratedHtml>>>,
    rss_feed: Arc<Mutex<::rss::ChannelBuilder>>,
    relative_links: Arc<Mutex<Vec<RelativeLink>>>,
    url_links: Option<Arc<Mutex<Sender<UrlLink>>>>,
}

impl Site {
    pub fn new(dest: PathBuf, root: PathBuf, url_sender: Option<Sender<UrlLink>>) -> Self {
        Self {
            dest,
            root,
            assets: Arc::new(Mutex::new(Vec::new())),
            pages: Arc::new(Mutex::new(Vec::new())),
            relative_links: Arc::new(Mutex::new(Vec::new())),
            url_links: url_sender.map(|s| Arc::new(Mutex::new(s))),
            rss_feed: Arc::new(Mutex::new(
                ::rss::ChannelBuilder::default()
                    .title("Eduardo's blog")
                    .link("https://eduardorittner.github.io")
                    .description("My blog")
                    .language("en-us".to_owned())
                    .docs("https://www.rssboard.org/rss-specification".to_owned())
                    .to_owned(),
            )),
        }
    }

    pub async fn build(self: Arc<Self>) -> Result<(), BuildError> {
        // TODO print what we are doing
        if !self.dest.exists() {
            tokio::fs::create_dir(&self.dest).await.unwrap();
        }

        for entry in WalkDir::new(&self.root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                let path = self.new_path(entry.path());
                if !path.exists() {
                    // TODO return error
                    tokio::fs::create_dir(path).await.unwrap();
                }
            } else {
                self.clone().process_file(entry.path()).await;
            }
        }

        self.validate_internal_links().await.unwrap();
        // TODO handle errors
        self.publish_rss().await.unwrap();
        println!("done building");
        Ok(())
    }

    pub async fn publish_rss(self: Arc<Self>) -> Result<(), BuildError> {
        // TODO handle errors
        let feed = self.rss_feed.lock().await;
        let channel = feed.build();
        let dest = self.dest.join(Path::new("rss.xml"));
        let file = File::create(dest).unwrap();
        channel.pretty_write_to(file, b' ', 2);
        Ok(())
    }

    pub async fn validate_internal_links(&self) -> Result<(), BuildError> {
        // TODO maybe return BuildError::InvalidLink(&Link) instead of cloning?
        // might matter more if we return a bunch of link errors
        let data = self.relative_links.lock().await;
        for item in data.iter() {
            let Link { link, file } = &item.0;
            if link.contains("#") {
                // TODO refactor method
                // TODO collect errors by declaring a build error variant that can contain more than 1 error
                heading_link_exists(&item.0)
                    .map_err(|_| BuildError::InvalidLink(item.0.clone()))?;
            } else {
                let path = file.parent().unwrap().join(Path::new(link));
                if !path.exists() {
                    return Err(BuildError::InvalidLink(item.0.clone()));
                }
            }
        }
        Ok(())
    }

    async fn process_file(self: Arc<Self>, entry: &Path) {
        if entry
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            == "md"
        {
            self.process_md(entry).await;
        } else {
            self.process_static(entry).await;
        }
    }

    async fn process_static(self: Arc<Self>, old_path: &Path) {
        // TODO return error
        let new_path = self.new_path(old_path);
        tokio::fs::copy(&old_path, &new_path).await.unwrap();
    }

    async fn process_md(self: Arc<Self>, old_path: &Path) {
        // TODO fix "publish: " with no date below title
        let new_path = self.new_path(old_path);
        let page = Page::new(old_path, new_path.strip_prefix(&self.dest).unwrap());

        if page.is_post() {
            let mut feed = self.rss_feed.lock().await;
            feed.item(new_item(&page));
        };

        let mut html_content = format_metadata(&page.metadata);

        let content = to_html(&page);

        html_content.push_str(&content);

        let dest_string = new_path.to_str().unwrap_or_default();
        let root_string = self.dest.to_str().unwrap_or_default();

        let (_, relative) = dest_string.split_once(root_string).unwrap_or_default();

        let depth = relative.chars().filter(|c| *c == '/').count() - 1;
        let prefix = if depth == 0 { "" } else { "../" };
        let html_header = format_header(&page.metadata.title, prefix);
        let html_navbar = format_navbar(prefix, page.category);
        let html_footer = format_footer();

        let html = html_header + &html_navbar + &html_content + &html_footer;

        let links = self.clone().process_links(&html, &new_path);
        let file_write = tokio::fs::write(&new_path, &html);
        let (_, e) = tokio::join!(links, file_write);

        if e.is_err() {
            panic!("Couldn't write to file: {:?} due to:\n{:?}", new_path, e)
        }
    }

    async fn process_links(self: Arc<Self>, source: &str, path: &Path) {
        let href = "href=";

        let mut source = source;

        while let Some(link_start) = source.find(href) {
            source = &source[link_start + href.len()..];

            let quote = source.chars().nth(0).unwrap();
            source = &source[1..];

            if let Some(end) = source.find(quote) {
                let link = &source[..end];

                match link {
                    external if link.contains("http") => {
                        // TODO exclude common links
                        if let Some(links) = &self.url_links {
                            let links = links.lock().await;
                            let _ = links
                                .send(UrlLink(Link {
                                    link: external.to_owned(),
                                    file: path.to_path_buf(),
                                }))
                                .await;
                        }
                    }
                    internal => {
                        let mut links = self.relative_links.lock().await;
                        links.push(RelativeLink(Link {
                            link: internal.to_owned(),
                            file: path.to_path_buf(),
                        }));
                    }
                }
            }
        }
    }

    fn new_path(&self, path: &Path) -> PathBuf {
        let new_path = path
            .to_str()
            .unwrap_or_default()
            .trim_start_matches(self.root.to_str().unwrap_or_default())
            .strip_prefix("/") // Strip '/' because of .join behavior on absolute paths
            .unwrap_or_default()
            .replace(".md", ".html")
            .to_string();
        self.dest.join(Path::new(&new_path))
    }
}

fn heading_link_exists(link: &Link) -> Result<(), ()> {
    let (file_path, heading) = match link.link.split_once("#") {
        // TODO maybe one fucntion that checks all links?
        None => unreachable!(),
        Some((file, relative_link)) if file.is_empty() => (link.file.as_path(), relative_link),
        Some((file, link)) => (Path::new(file), link),
    };

    // TODO return more complete error
    let contents = std::fs::read_to_string(file_path).unwrap();

    match contents.find(heading) {
        None => Err(()),
        Some(_) => Ok(()),
    }
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
