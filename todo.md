# Styling

- [X] Lighten link colors 
- [X] Increase spacing for headers inside articles
- [X] Add breaks along with article headers.
    - [X] Add breaks for page titles
    - [X] Surround every html header with <div class="break"> divs (using headingadapter?)
    - [X] Change color based on background (black on page titles, white on article headings)


# External features

- [X] Table of Contents
    - [ ] ToC with higher detail (currently only shows h2 headings), would be nice if it could show
    ordered and unordered list items, lower headings, etc.
- [X] Linkable headings
    - [X] Link icon
- [X] RSS
    - [X] Autodiscovery feature (rss link in metadata of every article)
    - [ ] Link with rss image in every footer
- [X] Syntax highlighting
- [ ] Vendor Mathjax script

## Optimization/Accesibility

- [X] html lang attribute
- [ ] Only include mathjax script in pages that use it
- [ ] Fallback font with font-display="swap"

# Internal features

- [ ] Parallelize tests
- [X] Make everything async! Mostly because validating online links takes a while so we could
    have some sort of shared buffer where online links are pushed and a thread does the checking
    while the files are being parsed and whatnot. Would be a fun challenge
- [ ] RSS expects dates in rfc2822, but hugo stored them in rfc3339. For now we do the conversion
    manually, but I would like to have some helper scripts to automatically insert the date in a
    file, and have all the dates in rfc 2822
- [ ] Validate rss file with any of the online rss validators

## Async!

I want to have the core logic be all async, since we can process .md files in parallel easily,
and the only thing that needs to run at the end are internal link validations, since we need
to have all the generated .html files to know whether the links are valid or not. So my plan is:

- Go through every file (walkdir from jwalk is already async)
- For every .md (in parallel)
    build the .html file (sync)
    find all the links (add each type to its equivalent queue)
    write to output folder
    add to rss feed
- While we are doing this, have a thread that consumes online links and tests them async
- By the end, we only need to check the collected internal links

## Correctness

Correctness doesn't matter that much, since I'm the only user and  any mistakes can be fixed
manually, however it's nice to practice this sort of stuff. I was thinking of doing something
where we only write to the target directory after having validated all the links and everything,
making builds atomic. This shouldn't be that hard, but might slow down builds (which are
pretty fast for now)

## Performance

Benchmark performance using criterion. Seems a bit more complicated to benchmark async functions

## Compilation time

Cold compilation times take quite a while, more than they should. It would be nice to make
them faster, probably by trimming some dependencies. The rss dependency, for example, uses
quite a lot of proc_macros which must be eating up a lot of the compilation time, and since
I only use some specific rss features, I could do it myself pretty easily.

A full build takes 90 seconds, and biggest compilation offenders are tokio and comrak.
Not surprising
