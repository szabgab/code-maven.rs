# Code Maven implemented in Rust

* Start as a static site generator but in a way that it will be easy to convert it into a real web site

* Go over all the .md files and call the page generator and then save the files.
* Go over all the extra pages (e.g. /archive) and create the pages for those.


* Read .md files with a header that looks like this (we'll define the exact fields later)

```
---
title: Title text
timestamp: 2015-10-11T12:30:01
todo:
  - Add another article extending on the topic
  - Add an article describing a prerequisite
---
```

Not yet implemented fields

```
indexes: open, File, read, readline, gets, while, each
status: show
books: ruby
author: szabgab
archive: 1
comments_disqus_enable: 0
```


## Testing

```
cargo test
```

## Local development

Generate the demo pages:

```
cargo run -- --root demo/ --pages demo/pages/ --outdir _site/
```

Assuming you have cloned https://github.com/szabgab/rust.code-maven.com/ next to this repository then you can also try:

```
cargo run -- --root ../rust.code-maven.com/ --pages ../rust.code-maven.com/pages/ --outdir _site/
```



Install [rustatic](https://rustatic.code-maven.com/) and run the following to view the generated site

```
rustatic --path _site/ --indexfile index.html --nice --port 3000
```
