use crate::*;
use comrak::{
    Options, PluginsBuilder, RenderPluginsBuilder, adapters, markdown_to_html_with_plugins, plugins,
};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
    dest: PathBuf,                          // Path to dest dir
    root: PathBuf,                          // Path to root dir
    assets: HashMap<PathBuf, AssetFile>,    // Key is the new path
    pages: HashMap<PathBuf, GeneratedHtml>, // Key is the new path
    rss_feed: ::rss::ChannelBuilder,
    relative_links: Vec<RelativeLink>,
    url_links: Option<Vec<UrlLink>>,
}

impl Default for Site {
    fn default() -> Self {
        let root = PathBuf::from("../src");
        let to = PathBuf::from("../output");
        Site::new(to, root, None)
    }
}

impl Site {
    pub fn new(dest: PathBuf, root: PathBuf, url_sender: Option<Vec<UrlLink>>) -> Self {
        Self {
            dest,
            root,
            assets: HashMap::new(),
            pages: HashMap::new(),
            relative_links: Vec::new(),
            url_links: url_sender,
            rss_feed: ::rss::ChannelBuilder::default()
                .title("Eduardo's blog")
                .link("https://eduardorittner.github.io")
                .description("My blog")
                .language("en-us".to_owned())
                .docs("https://www.rssboard.org/rss-specification".to_owned())
                .to_owned(),
        }
    }

    pub fn build(dest: PathBuf, root: PathBuf) -> Result<(), BuildError> {
        let mut site = Site::new(dest, root, None);

        site.generate().unwrap();
        // Invalid links are only warnings for now
        if let Err(e) = site.validate_internal_links() {
            eprintln!("{:?}", e);
        }

        site.commit_build()
    }

    /// Generates all html pages and validates internal links
    pub fn generate(&mut self) -> Result<(), BuildError> {
        println!("Starting build");

        if !self.dest.exists() {
            std::fs::create_dir(&self.dest).map_err(|e| BuildError::IoError(e))?;
        }

        for entry in WalkDir::new(&self.root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                let path = self.new_path(entry.path());
                if !path.exists() {
                    std::fs::create_dir(path).map_err(|e| BuildError::IoError(e))?;
                }
            } else {
                let _ = self.process_file(entry.path());
            }
        }
        Ok(())
    }

    /// Writes all the changes to the filesystem.
    pub fn commit_build(&mut self) -> Result<(), BuildError> {
        println!("Commiting changes");
        self.publish_rss().unwrap();

        self.pages.iter().for_each(|(_path, content)| {
            std::fs::write(&content.to, &content.content).unwrap();
        });

        self.assets.iter().for_each(|(_path, asset)| {
            std::fs::copy(&asset.from, &asset.to).unwrap();
        });

        Ok(())
    }

    pub fn publish_rss(self: &mut Self) -> Result<(), BuildError> {
        let channel = self.rss_feed.build();
        let dest = self.dest.join(Path::new("rss.xml"));
        let file = File::create(dest).map_err(|e| BuildError::IoError(e))?;
        channel.pretty_write_to(file, b' ', 2).unwrap();
        Ok(())
    }

    pub fn validate_internal_links(&self) -> Result<(), BuildError> {
        println!("Validating internal links");

        let mut invalid_links = InvalidLinks(Vec::new());

        let data = &self.relative_links;
        for item in data.iter() {
            if let Err(_) = self.heading_link_exists(&item.0) {
                invalid_links.0.push(item.0.clone());
            }
        }

        if !invalid_links.0.is_empty() {
            Err(BuildError::InvalidLinks(invalid_links))
        } else {
            Ok(())
        }
    }

    fn process_file(&mut self, entry: &Path) -> Result<(), BuildError> {
        if entry
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            == "md"
        {
            self.process_md(entry)
        } else {
            self.process_static(entry);
            Ok(())
        }
    }

    fn process_static(&mut self, old_path: &Path) {
        let new_path = self.new_path(old_path);

        self.assets.insert(
            canonical(&new_path),
            AssetFile {
                to: new_path,
                from: old_path.to_owned(),
            },
        );
    }

    fn process_md(&mut self, old_path: &Path) -> Result<(), BuildError> {
        let mut new_path = self.new_path(old_path);
        new_path.set_extension("html");
        let page = Page::new(old_path, new_path.strip_prefix(&self.dest).unwrap());

        if page.metadata.draft {
            return Ok(());
        }

        if page.is_post() {
            self.rss_feed.item(new_item(&page));
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

        self.process_links(&html, &new_path);

        self.pages.insert(
            canonical(&new_path),
            GeneratedHtml {
                to: new_path.to_owned(),
                from: old_path.to_owned(),
                content: html,
            },
        );

        Ok(())
    }

    fn process_links(&mut self, source: &str, path: &Path) {
        let href = "href=";

        let mut source = source;

        while let Some(link_start) = source.find(href) {
            source = &source[link_start + href.len()..];

            let quote = source.chars().nth(0).unwrap();
            source = &source[1..];

            if let Some(end) = source.find(quote) {
                let link = &source[..end];

                match link {
                    // TODO: Do we still need to external links?
                    external if link.contains("http") => {
                        let link = UrlLink(Link {
                            link: external.to_owned(),
                            file: path.to_path_buf(),
                        });

                        match self.url_links.as_mut() {
                            None => self.url_links = Some(vec![link]),
                            Some(links) => links.push(link),
                        }
                    }
                    internal => {
                        self.relative_links.push(RelativeLink(Link {
                            link: internal.to_owned(),
                            file: path.to_path_buf(),
                        }));
                    }
                }
            }
        }
    }

    fn heading_link_exists(self: &Self, link: &Link) -> Result<(), ()> {
        // TODO deal with full paths everywhere and only convert to relative when necessary?
        let (file_path, heading) = match link.link.split_once("#") {
            // Link is only path
            None => (Path::new(&link.link), ""),
            // Link is only heading (file is the current one)
            Some((file, heading)) if file.is_empty() => {
                let path = canonical(Path::new(&link.file));
                if let Some(page) = self.pages.get(&path) {
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

        if let Some(page) = self.pages.get(&abs_path) {
            match page.content.find(heading) {
                None => Err(()),
                Some(_) => Ok(()),
            }
        } else if let Some(_) = self.assets.get(&abs_path) {
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
    options.extension.footnotes = true;

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
