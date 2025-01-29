use crate::page::*;

pub fn table_of_contents(source: String) -> String {
    let header_ids: Vec<&str> = source
        .split("<h2 id=\"")
        .skip(1)
        .map(|id| {
            &id[..id.find('\"').unwrap_or_else(|| panic!("Expected a final '\"' after parsing h2 id in:\n{id}"))]
        })
        .collect();

    format_toc(header_ids) + &source
}

pub fn format_toc(titles: Vec<&str>) -> String {
    if titles.is_empty() {
        return String::new();
    }

    let mut toc = String::with_capacity(1024);

    toc.push_str("<details>");
    toc.push_str("<summary>Table of Contents</summary>");
    toc.push_str(r##"<ul id="table-of-contents" class="section-toc">"##);
    for title in titles {
        let visible_title = title.replace("-", " ");
        toc.push_str(r##"<li class="toc-entry toc-h2">"##);
        toc.push_str(&format!(
            r##"<a href="#{}">{}</a>"##,
            &title, &visible_title
        ));
        toc.push_str("</li>");
    }
    toc.push_str("</ul>");
    toc.push_str("</details>");
    toc
}

pub fn format_header(title: &str, root: &str) -> String {
    format!(
        "<!doctype html>\
    <html lang=\"en-US\">\
    <head> \
    <title>{}</title> \
    <link href=\"{}style.css\" rel=\"stylesheet\" type=\"text/css\" media=\"all\"> \
    <link href='https://fonts.googleapis.com/css?family=Fira Mono' rel='stylesheet'> \
    <link rel=\"alternate\" type=\"application/rss+xml\" title=\"RSS\"\
    href=\"https://eduardorittner.github.io/rss.xml\">\
    <meta charset=\"UTF-8\"> \
    <script type=\"text/x-mathjax-config\"> \
    MathJax.Hub.Config({{ \
    tex2jax: {{inlineMath: [['$','$'], ['\\\\(','\\\\)']]}} \
    }}); \
    </script> \
    <script type=\"text/javascript\" \
    src=\"https://cdn.mathjax.org/mathjax/latest/MathJax.js?config=TeX-AMS-MML_HTMLorMML\"> \
    </script> \
    </head> \
    ",
        title, root
    )
}

pub fn format_navbar(prefix: &str, kind: Category) -> String {
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

pub fn format_footer() -> String {
    "</article></body></html>".to_string()
}

pub fn format_metadata(metadata: &Metadata) -> String {
    let mut title = format!(
        "<article id=\"post\">\
        <div class=\"stack\">\
        <div class=\"heading-container\">\
        <div class=\"break\"></div>\
        <h1>{}</h1>\
        <div class=\"break\"></div>\
        </div>",
        metadata.title
    );
    if let Some((date, _)) = metadata.date.clone().unwrap_or_default().split_once('T') {
        let date = format!("<span class=\"date\">Published: {}</span>", date);
        title.push_str(&date);
    }
    title.push_str("</div>");
    title
}
