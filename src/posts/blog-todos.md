+++
title = 'Building my own Static Site Generator in Rust'
date = 2024-12-05T12:06:09-03:00
draft = false
+++

# But first, context

I originally started this blog using [Hugo](https://gohugo.io/), a static-site generator (SSG) written in Go which makes building static sites from markdown files very easy. The problem with hugo is that it's a very complicated piece of software, with lots of intricacies, it's own templating engine, folder structure and so on. It also moves fast, too fast for my taste. It's probably a very good SSG, I just never really got the hang of it, I guess.

So I decided to ditch Hugo and start my own SSG in rust after reading how NRK builds his site with a [Makefile](https://nrk.neocities.org/articles/site-open-source). Since, if he can do it with a 30 line makefile and some bash scripts, why can't I do it in 300 lines of rust? It's nice to have something small and simple, which I understand completely and can change and add whatever I want pretty easily, it's a very different feeling from using Hugo, which always felt like somewhat of a black box to me.

After the initial weeks I've now got something that mostly works, and my current priority is to add features to the site, as well as polishing the overall look and feel of it. This post is a way for me to write down what I want, so I don't forget it, since what usually happens is I think of a cool feature right before I sleep, then I sleep, then by the next morning I can't remember what it was.

# Features I want to have

Right now things are pretty simple but there are some features which I'd really like to add, taken from Daniel's [Microfeatures I Love in Blogs and Personal Websites](https://danilafe.com/blog/blog_microfeatures/) post, which I enjoyed.

## Table of contents

This one is pretty self-explanatory, I'd like for my posts to have a ToC at the top. I like being able to view all the headings and subheadings, as this provides a nice overview of the post's structure, and makes navigating them easier. If possible, I also want there to be some visual indication of where I currently am while reading the articles.

## Linkable Headings

Combined with ToCs, this feature just makes reading a post easier, and is pretty simple to implement from what I gather. This would make it possible to link to specific headings inside a post, not just to a post itself.

## Syntax highlighting

Since I write mostly about coding, source code is bound to appear in my posts, and source code without syntax highlighting just looks ugly. It also makes it harder to read, in my opinion.

## RSS feed

An RSS feed makes it easy to read my posts from outside the browser, and get notifications for every new post. From what I gather it's a simple XML file with all the posts, so it shouldn't be that hard to do.

## Better styling

I'm pretty new to this html+css thing, and there are a lot of rough edges I want to polish still: Headings are not currently centered; Post titles are not quite right yet; The mobile experience could be better; etc. etc.

# Quality of life features

There are also some internal features which I want to add. These should not really change the final result, but are only intended to make my life easier, as well as to gain some confidence with rust.

## My blog stack

The rust code that builds my site is on the same repo as my [blog](https://github.com/eduardorittner/eduardorittner.github.io), and is a crate named (very creatively) builder. Right now all it does is:

1. walk all directories recursively starting from `src/`
2. For every non-md file, copy it to `/output` as is
3. For every md file:
    1. Parse its metadata
    2. Construct a `Page` type, which contains its category, metadata and content
    3. Convert it to html with comrak
    4. Add some more html which is common amongst all pages (header, footer, navbar, etc.)
    5. Save the generated html in `/output`

# What I want

As I've already alluded to, my crate runs both locally and on github actions. These are 2 very different environments, and they also have different goals. For every push, github has to download all dependencies, build the crate from scratch, and run it to produce the final result, while locally I'm doing mostly warm builds which are almost instantaneous. Another thing is that I would like to keep my git history as clean as possible, so I'd rather test everything locally and only then push the changes.

What this means is that when I run my code locally, I'd like it to test all the links and be as thorough as possible. And for the code that runs on github, it ideally should only do the bare minimum to build the final version, which means as little dependencies and code as possible. This will be achieved via conditional compilation, probably.

# Final points

Doing this has been a really fun and educational experience so far, and I hope it will stay that way. I hope I'll be able to improve my site, while also maintaining a somewhat regular influx of content. If you somehow stumble upon here and have any suggestions, tips or opinions, I'd really like to hear them! You can find me on github or email me at eduardorittnerc@gmail.com.

