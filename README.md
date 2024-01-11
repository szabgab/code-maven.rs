# Code Maven implemented in Rust

* Start as a static site generator but in a way that it will be easy to convert it into a real web site

* Go over all the .md files and call the page generator and then save the files.
* Go over all the extra pages (e.g. /archive) and create the pages for those.


* Read .md files with a header that looks like this (we'll define the exact fields later)


## Installation

* Install Rust
* Clone https://github.com/szabgab/code-maven.rs
* cd code-maven.rs


* cargo install --path .   # TODO: we need to integrate the templates in order to make this work.


## Setup web site

* Create a new folder
* In the folder create a file called `config.yaml` based on the one in the `demo` folder of the [repository](https://github.com/szabgab/code-maven.rs)
* Create a folder called `pages`.
* In the `pages` folder create a file called `index.md`. See the format below.
* Run `code-maven`

* Open issues on the [project](https://github.com/szabgab/code-maven.rs) if something is unclear or does not work as you expected.

## Format

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

Install [pre-commit](https://pre-commit.com/) and run `pre-commit install` to configure it for this repo.

* In the `demo` folder we have the files of a web site.
* In the `demo_site` folder we have the generated version of the demo site.
* Running the `tests.sh` will generated the demo site and compare it to the stored version.

Generate the demo pages:

```
cargo run --bin code-maven -- web --root demo/ --outdir _site/
```

We can also explicitly say where the pages are, but so far we did not need that.

```
cargo run --bin code-maven -- web --root demo/ --pages demo/pages/ --outdir _site/
```

Assuming you have cloned https://github.com/szabgab/rust.code-maven.com/ next to this repository then you can also try:

```
cargo run --bin code-maven -- web --root ../rust.code-maven.com/ --outdir _site/
```



Install [rustatic](https://rustatic.code-maven.com/) and run the following to view the generated site

```
rustatic --path _site/ --indexfile index.html --nice --port 3000
```


## Generate list of recent posts

With links to post from the last 3 days (actually 3*24 hours)

```
cargo run -- --root ../rust.code-maven.com/  --outdir _site/ --email 3
```
