use crate::page::*;

pub fn format_header(title: &str, root: &str) -> String {
    format!(
        "<html>\
    <head> \
    <title>{}</title> \
    <link href=\"{}style.css\" rel=\"stylesheet\" type=\"text/css\" media=\"all\"> \
    <link href='https://fonts.googleapis.com/css?family=Fira Mono' rel='stylesheet'> \
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
        <div class=\"container\">\
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
