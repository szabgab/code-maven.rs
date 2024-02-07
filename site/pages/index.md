---
title: Code Maven Static Site Generator
timestamp: 2023-11-08T11:30:01
published: true
description: Static Site Generator
---

# Code Maven SSG

This is the static site generator behind the [Rust Maven](https://rust.code-maven.com/) web site and a [couple of other sites](/where-is-it-used).

## Linux:

Download [code-maven](/code-maven) and make it executable

```
chmod +x code-maven
```

## Setup

Run

```
code-maven new --root path-to-new-site
```

This will create a folder to hold the site.

* In the folder it will create a file called `config.yaml` based on the one in the `demo` folder of the [repository](https://github.com/szabgab/code-maven.rs) or based on [this config.yaml](https://github.com/szabgab/code-maven.rs/blob/main/site/config.yaml).

It will create a folder called `pages` with a number of pages:

A file called `index.md` based on [this file](https://raw.githubusercontent.com/szabgab/code-maven.rs/main/site/pages/index.md). The field in the front-matter, at the top are the important bit. See the format below.

```
cd path-to-new-site
```

Run `code-maven web`.   It will generated the site in the `_site` folder.

* If something is unclear or does not work as you expected open issues on the [project](https://github.com/szabgab/code-maven.rs).

## Format

```
---
title: Title text
timestamp: 2015-10-11T12:30:01
author:
published: false
tags:
  - web
  - rust
todo:
  - Add another article extending on the topic
  - Add an article describing a prerequisite
---
```


## View the site locally

You can open the individual pages in the `_site` folder using your browser or you can run a mini web server serving static pages.

For the latter download [rustatic](https://rustatic.code-maven.com/) and run

```
rustatic --path _site/ --nice --indexfile index.html --port 500
```

## GitHub pages

In order to setup a site on GitHub pages crate a file called `/.github/workflows/gh-pages.yml`  (the folder matters, the actual name of the file can be anything as long as the extension is `.yml` or `.yaml`.)

The content of the file:

![](include/gh-pages.yml)

