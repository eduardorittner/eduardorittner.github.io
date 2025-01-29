+++
title = 'Overengineering my homemade SSG for fun and (no) profit'
date = 2023-09-21T14:47:56-03:00
draft = false
+++

I've recently been working on my homemade SSG written from scratch in rust. I've had something that works for some months now, since SSG ares conceptually pretty simple (until you start adding templating languages, templates, styles and all that), you take some markdown files, convert them to html, add a header, footer, and put them in a new directory.
However, as most programmers, I have the desire to ma

# Modelling the problem

Inside the SSG pipeline, there are operations which can be done concurrently and operations that cannot. Concurrent operations are currently: converting .md files to .html, adding items to the rss feed and checking if url links are valid. Serial operations which must occur at the end are: generating the final rss.xml file and checking if internal links are valid, since we need all the .html files to have been generated.

We are going to model this using Rust's excellent type-system to guide us, going bottom-up:

## Links

First, we'll model links, which must be `Send + Sync` in order to be moved or sent accross threads, and can be either url links or internal (relative) links

```rust
struct Link {
	link: String,
	file: PathBuf,
}

struct RelativeLink(Link);

struct UrlLink(Link);

struct InvalidLink(Link);
```

We use `String` instead of `&str` since they own their data and so can be sent across thread, ditto for `PathBuf`. Another design choice is to have two different struct types, instead of one struct differentiated by a `LinkKind` enum, since they are not really interchangeable and will always be handled differently, having different types for each makes the distinction between them clearer.
We'll also have an error type which represents an invalid link, which is a simple wrapper around `Link`. We could have something more sophisticated which would wrap additional errors for more context and information, but for our case this is enough. Note that for this to implement the `Error` trait we must implement `Debug` for it.

## Files

Our files start out as markdown files which are read to a String, then converted to html and then written to the output directory. We also want access to the generated html when checking whether relative links are valid or not, we can do this a few ways:
1. Save the html as soon as we generate it and then drop it, if we find a relative link which points to the html, see if it exists (for non-heading links) by checking the file-system or read it from disk (for heading links)
2. Save the html as soon as we generate it but keep the contents in RAM as well, if we find a relative link which points to the html, we already have the contents in RAM ready to go.
3. Keep the html in RAM and only save them to disk after having validated all the links.

The third approach has the nice benefit of being "pure", i.e. it only commits the files to disk if the whole build is valid, while the first and second approaches always commit to disk, but may error out if the build is invalid. Since we want this site runs on Github Pages, I believe the third option is the best. It is simpler and more correct, in my opinion.

```rust
struct GeneratedHtml {
	from: PathBuf,
	to: PathBuf,
	content: String,
}
```

This contains the origin path to the .md, the destination path to the .html, and the internal contents.

We also have static files, such as images, .css files, and what not. These are simply stored as a  pair of `PathBuf`s.

```rust
struct AssetFile {
	from: PathBuf,
	to: PathBuf,
}
```

## Site

With all of that we can finally model our site struct:
```rust
struct Site {
    dest: PathBuf, // Path to dest dir
    root: PathBuf, // Path to root dir
    assets: Arc<Mutex<Vec<AssetFile>>>,
    pages: Arc<Mutex<Vec<GeneratedHtml>>>,
    rss_feed: Arc<Mutex<::rss::ChannelBuilder>>,
    relative_links: Arc<Mutex<Vec<RelativeLink>>>,
    url_links: Option<Arc<Mutex<Sender<UrlLink>>>>,
}
```

We can see that we have two paths pointing to the source directory and output directory, a list of `AssetFile`s, a list of `GeneratedHtml`s, an rss channel builder (taken from the rss rust crate), a list of `RelativeLink`s and a `Sender` of `UrlLink`s.

Note that we want url_links to be a channel with multiple senders and only one receiver. Every instance of Site has a copy of the sender, and a separate task only listens to the channel and checks whatever links it's passed

# Doing the work

Now that we have our types written down we can start to actually code our intended behavior! We start off simple, by implementing a function that returns a new instance of `Site`.

```rust
fn new(dest: PathBuf, root: PathBuf, url_sender: Option<Sender<UrlLink>>) -> Self {
	Self {
		dest,
		root,
		assets: Arc::new(Mutex::new(Vec::new())),
		pages: Arc::new(Mutex::new(Vec::new())),
		relative_links: Arc::new(Mutex::new(Vec::new())),
		url_links: url_sender.map(|s| Arc::new(Mutex::new(s))),
		rss_feed: Arc::new(Mutex::new(::rss::ChannelBuilder::default())),
	}
}
```

Next we'll define our entry point function, the one call that generates our resulting site:

```rust
async fn build(self: Self) -> Result<(), BuildError> {

}
```