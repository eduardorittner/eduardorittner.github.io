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
    - [ ] Autodiscovery feature (rss link in metadata of every article)
    - [ ] Link with rss image in every footer
- [X] Syntax highlighting
- [ ] Vendor Mathjax script

## Optimization/Accesibility

- [X] html lang attribute
- [ ] Only include mathjax script in pages that use it
- [ ] Fallback font with font-display="swap"

# Internal features

- [ ] Parallelize tests
- [ ] Make everything async! Mostly because validating online links takes a while so we could
    have some sort of shared buffer where online links are pushed and a thread does the checking
    while the files are being parsed and whatnot. Would be a fun challenge
- [ ] RSS expects dates in rfc2822, but hugo stored them in rfc3339. For now we do the conversion
    manually, but I would like to have some helper scripts to automatically insert the date in a
    file, and have all the dates in rfc 2822
- [ ] Validate rss file with any of the online rss validators
