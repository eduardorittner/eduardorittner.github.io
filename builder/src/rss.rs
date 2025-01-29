use crate::Page;
use rss::{ChannelBuilder, ItemBuilder};

pub fn new_rss(items: Vec<rss::Item>) -> rss::Channel {
    ChannelBuilder::default()
        .title("Eduardo's blog")
        .link("https://eduardorittner.github.io")
        .description("My blog")
        .language("en-us".to_owned())
        .docs("https://www.rssboard.org/rss-specification".to_owned())
        .items(items)
        .build()
}

pub fn new_item(page: &Page) -> rss::Item {
    let mut link = "https://eduardorittner.github.io/".to_owned();
    link.push_str(page.path.to_str().unwrap());

    let mut item = ItemBuilder::default();

    item.title(page.metadata.title.clone())
        .link(link.to_owned());

    if let Some(date) = &page.metadata.date {
        item.pub_date(date.clone()).build()
    } else {
        item.build()
    }
}
