use crate::*;
use comrak::{
    adapters, markdown_to_html_with_plugins, plugins, Options, PluginsBuilder, RenderPluginsBuilder,
};
use reqwest;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use walkdir::WalkDir;

pub struct ExternalLinkValidator(pub Receiver<UrlLink>);

impl ExternalLinkValidator {
    pub async fn run_validator(mut self) -> Result<(), BuildError> {
        let mut errors = InvalidLinks(Vec::new());

        let mut requests = Vec::new();
        let mut links = Vec::new();

        while let Some(link) = self.0.recv().await {
            links.push(link.clone());
            requests.push(tokio::spawn(reqwest::get(link.0.link)));
        }

        for (req, link) in requests.into_iter().zip(links) {
            if let Err(_) = req.await {
                errors.0.push(link.0);
            }
        }

        if errors.0.is_empty() {
            Ok(())
        } else {
            Err(BuildError::InvalidLinks(errors))
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

impl Default for Site {
    fn default() -> Self {
        let root = PathBuf::from("../src");
        let to = PathBuf::from("../output");
        Site::new(to, root, None)
    }
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

    pub async fn build_with_url_validator(dest: PathBuf, root: PathBuf) -> Result<(), BuildError> {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        let validator = ExternalLinkValidator(rx);
        // TODO handle error
        let validator = tokio::spawn(validator.run_validator());

        {
            Site::build(dest, root, Some(tx)).await?;
        }

        println!("Checking url links");
        // Wait for external url validator to finish before exiting
        validator.await.unwrap()
    }

    pub async fn build(
        dest: PathBuf,
        root: PathBuf,
        url_sender: Option<Sender<UrlLink>>,
    ) -> Result<(), BuildError> {
        let site = Arc::new(Site::new(dest, root, url_sender));
        println!("Starting build");

        if !site.dest.exists() {
            tokio::fs::create_dir(&site.dest)
                .await
                .map_err(|e| BuildError::IoError(e))?;
        }

        for entry in WalkDir::new(&site.root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                let path = site.new_path(entry.path());
                if !path.exists() {
                    tokio::fs::create_dir(path)
                        .await
                        .map_err(|e| BuildError::IoError(e))?;
                }
            } else {
                site.clone().process_file(entry.path()).await?;
            }
        }

        site.clone().publish_rss().await?;
        println!("Done building");

        println!("Checking internal links");

        site.validate_internal_links().await?;

        println!("Internal links OK");

        Ok(())
    }

    pub async fn publish_rss(self: Arc<Self>) -> Result<(), BuildError> {
        let feed = self.rss_feed.lock().await;
        let channel = feed.build();
        let dest = self.dest.join(Path::new("rss.xml"));
        let file = File::create(dest).map_err(|e| BuildError::IoError(e))?;
        channel.pretty_write_to(file, b' ', 2);
        Ok(())
    }

    pub async fn validate_internal_links(&self) -> Result<(), BuildError> {
        let mut invalid_links = InvalidLinks(Vec::new());

        let data = self.relative_links.lock().await;
        for item in data.iter() {
            if let Err(_) = heading_link_exists(&item.0) {
                invalid_links.0.push(item.0.clone());
            }
        }
        Ok(())
    }

    async fn process_file(self: Arc<Self>, entry: &Path) -> Result<(), BuildError> {
        if entry
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            == "md"
        {
            self.process_md(entry).await
        } else {
            self.process_static(entry).await
        }
    }

    async fn process_static(self: Arc<Self>, old_path: &Path) -> Result<(), BuildError> {
        let new_path = self.new_path(old_path);
        tokio::fs::copy(&old_path, &new_path)
            .await
            .map_err(|e| BuildError::IoError(e))?;
        Ok(())
    }

    async fn process_md(self: Arc<Self>, old_path: &Path) -> Result<(), BuildError> {
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

        e.map_err(|e| BuildError::IoError(e))?;
        Ok(())
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
        None => (link.file.as_path(), link.link.as_str()),
        Some((file, relative_link)) if file.is_empty() => (link.file.as_path(), relative_link),
        Some((file, link)) => (Path::new(file), link),
    };

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
