use crate::*;
use async_std;
use async_std::channel::{Receiver, Sender};
use async_std::prelude::*;
use async_std::sync::Mutex;
use async_std::task::spawn;
use comrak::{
    adapters, markdown_to_html_with_plugins, plugins, Options, PluginsBuilder, RenderPluginsBuilder,
};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

pub struct ExternalLinkValidator(pub Receiver<UrlLink>);

impl ExternalLinkValidator {
    pub async fn run_validator(mut self) -> Result<(), BuildError> {
        let mut errors = InvalidLinks(Vec::new());

        let mut requests = Vec::new();
        let mut links = Vec::new();

        while let Ok(link) = self.0.recv().await {
            links.push(link.clone());
            requests.push(async_std::task::spawn(surf::get(link.0.link)));
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GeneratedHtml {
    to: PathBuf,
    from: PathBuf,
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AssetFile {
    to: PathBuf,
    from: PathBuf,
}

pub struct Site {
    dest: PathBuf,                                      // Path to dest dir
    root: PathBuf,                                      // Path to root dir
    assets: Arc<Mutex<HashMap<PathBuf, AssetFile>>>,    // Key is the new path
    pages: Arc<Mutex<HashMap<PathBuf, GeneratedHtml>>>, // Key is the new path
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
            assets: Arc::new(Mutex::new(HashMap::new())),
            pages: Arc::new(Mutex::new(HashMap::new())),
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
        let (tx, rx) = async_std::channel::unbounded();
        let validator = ExternalLinkValidator(rx);
        // TODO handle error
        let validator = async_std::task::spawn(validator.run_validator());

        let site = Arc::new(Site::new(dest, root, Some(tx)));
        site.clone().generate().await?;
        site.validate_internal_links().await?;

        println!("Checking url links");

        let mut site = Arc::try_unwrap(site).ok().unwrap();
        // Drop the sender to close the channel
        site.url_links = None;

        // Wait for external url validator to finish before exiting
        validator.await?;

        site.commit_build().await
    }

    pub async fn build(dest: PathBuf, root: PathBuf) -> Result<(), BuildError> {
        let site = Arc::new(Site::new(dest, root, None));

        site.clone().generate().await?;
        site.validate_internal_links().await?;

        let site = Arc::try_unwrap(site).ok().unwrap();

        site.commit_build().await
    }

    /// Generates all html pages and validates internal links
    pub async fn generate(self: Arc<Self>) -> Result<(), BuildError> {
        println!("Starting build");

        if !self.dest.exists() {
            async_std::fs::create_dir(&self.dest)
                .await
                .map_err(|e| BuildError::IoError(e))?;
        }

        for entry in WalkDir::new(&self.root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                let path = self.new_path(entry.path());
                if !path.exists() {
                    async_std::fs::create_dir(path)
                        .await
                        .map_err(|e| BuildError::IoError(e))?;
                }
            } else {
                self.clone().process_file(entry.path()).await?;
            }
        }
        Ok(())
    }

    /// Writes all the changes to the filesystem.
    /// This method should be called with only one `Site` instance alive
    /// since it takes ownership of the internal `Vec`s of `GeneratedHtml`
    /// and `AssetFile`s, as well as drops the `Site` instance upon exit.
    pub async fn commit_build(mut self: Self) -> Result<(), BuildError> {
        println!("Commiting changes");
        self.publish_rss().await?;

        let pages = Arc::try_unwrap(self.pages)
            .ok()
            .expect("There should only be 1 instance of `Site` at this point")
            .into_inner();

        let assets = Arc::try_unwrap(self.assets)
            .ok()
            .expect("There should only be 1 instance of `Site` at this point")
            .into_inner();

        let page_jobs: Vec<_> = pages
            .into_iter()
            .map(|(_, page)| spawn(async_std::fs::write(page.to, page.content.clone())))
            .collect();

        let asset_jobs: Vec<_> = assets
            .into_iter()
            .map(|(_, asset)| spawn(async_std::fs::copy(asset.from.clone(), asset.to.clone())))
            .collect();

        for job in page_jobs {
            job.await.map_err(|e| BuildError::IoError(e))?;
        }

        for job in asset_jobs {
            job.await.map_err(|e| BuildError::IoError(e))?;
        }

        Ok(())
    }

    pub async fn publish_rss(self: &mut Self) -> Result<(), BuildError> {
        let feed = self.rss_feed.lock().await;
        let channel = feed.build();
        let dest = self.dest.join(Path::new("rss.xml"));
        let file = File::create(dest).map_err(|e| BuildError::IoError(e))?;
        channel.pretty_write_to(file, b' ', 2);
        Ok(())
    }

    pub async fn validate_internal_links(&self) -> Result<(), BuildError> {
        println!("Validating internal links");

        let mut invalid_links = InvalidLinks(Vec::new());

        let data = self.relative_links.lock().await;
        for item in data.iter() {
            if let Err(_) = self.heading_link_exists(&item.0).await {
                invalid_links.0.push(item.0.clone());
            }
        }

        if !invalid_links.0.is_empty() {
            Err(BuildError::InvalidLinks(invalid_links))
        } else {
            Ok(())
        }
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
            self.process_static(entry).await;
            Ok(())
        }
    }

    async fn process_static(self: Arc<Self>, old_path: &Path) {
        let new_path = self.new_path(old_path);
        let mut data = self.assets.lock().await;

        data.insert(
            canonical(&new_path),
            AssetFile {
                to: new_path,
                from: old_path.to_owned(),
            },
        );
    }

    async fn process_md(self: Arc<Self>, old_path: &Path) -> Result<(), BuildError> {
        let mut new_path = self.new_path(old_path);
        new_path.set_extension("html");
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

        self.clone().process_links(&html, &new_path).await;

        let mut data = self.pages.lock().await;

        data.insert(
            canonical(&new_path),
            GeneratedHtml {
                to: new_path.to_owned(),
                from: old_path.to_owned(),
                content: html,
            },
        );

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

    async fn heading_link_exists(self: &Self, link: &Link) -> Result<(), ()> {
        let pages = self.pages.lock().await;
        let assets = self.assets.lock().await;

        // TODO deal with full paths everywhere and only convert to relative when necessary?
        let (file_path, heading) = match link.link.split_once("#") {
            // Link is only path
            None => (Path::new(&link.link), ""),
            // Link is only heading (file is the current one)
            Some((file, heading)) if file.is_empty() => {
                let path = canonical(Path::new(&link.file));
                if let Some(page) = pages.get(&path) {
                    return match page.content.find(heading) {
                        None => Err(()),
                        Some(_) => Ok(()),
                    };
                } else {
                    return Err(());
                }
            }
            // Link is path + heading
            Some((file, link)) => (Path::new(file), link),
        };

        let path_to_linker = self.root.join(link.file.parent().unwrap());
        let path_to_linkee = path_to_linker.join(file_path);
        let abs_path = canonical(&path_to_linkee);

        if let Some(page) = pages.get(&abs_path) {
            match page.content.find(heading) {
                None => Err(()),
                Some(_) => Ok(()),
            }
        } else if let Some(_) = assets.get(&abs_path) {
            // Can't have internal links to assets
            assert!(heading.is_empty());
            Ok(())
        } else {
            println!("{abs_path:?} not found");
            Err(())
        }
    }

    fn new_path(&self, path: &Path) -> PathBuf {
        let new_path = path.strip_prefix(&self.root).unwrap().to_owned();
        self.dest.join(&new_path)
    }
}

fn canonical(path: &Path) -> PathBuf {
    if !path.exists() {
        let _ = std::fs::write(path, "");
    };

    path.canonicalize().unwrap()
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
