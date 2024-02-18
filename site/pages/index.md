---
title: Code Maven Static Site Generator
timestamp: 2023-11-08T11:30:01
published: true
description: Static Site Generator
---

This is the static site generator behind the [Rust Maven](https://rust.code-maven.com/) web site and a [couple of other sites](/where-is-it-used).

## Linux:

Download [code-maven](/code-maven) and make it executable

```
chmod +x code-maven
```

## Setup - Create new site locally

Run

```
code-maven new --root path-to-new-site
```

This will create a folder to hold the site. This is the layout of the folder:

```
$ tree
.
├── authors
│   └── foobar.md
├── config.yaml
├── .gitignore
└── pages
    ├── about.md
    └── index.md
```

* In the folder it will create a file called `config.yaml` based on the one in the `demo` folder of the [repository](https://github.com/szabgab/code-maven.rs) or based on [this config.yaml](https://github.com/szabgab/code-maven.rs/blob/main/site/config.yaml). Explanation about the fields will be included in the skeleton site and can be also seen [here](https://github.com/szabgab/code-maven.rs/blob/main/test_cases/skeleton/config.yaml).

It will create a folder called `pages` with a number of pages:

A file called `index.md` based on [this file](https://raw.githubusercontent.com/szabgab/code-maven.rs/main/site/pages/index.md). The field in the front-matter, at the top are the important bit. See the format below.

The `authors` folder will hold information about each author. As an example an author called `foobar` is used for the default pages and a file is created for this author.

## Generate the web site

```
cd path-to-new-site
```

Run `code-maven web`.   It will generated the site in the `_site` folder.

* If something is unclear or does not work as you expected open issues on the [project](https://github.com/szabgab/code-maven.rs).

## Format

Each page is generated from a Markdown file located in the `pages` folder.

### Front-Matter

At the top of each file there is some meta-information called the [Front-Matter](/front-matter).

After this front-matter feel free to write any Markdown.

### Special syntax

There is some [Special Syntax](/special-syntax) - tags you can embed in the Markdown files.

## View the site locally

You can open the individual pages in the `_site` folder using your browser or you can run a mini web server serving static pages.

For the latter download [rustatic](https://rustatic.code-maven.com/) and run

```
rustatic --path _site/ --nice --indexfile index.html --port 500
```

## Generate list of recent posts

With links to post from the last 3 days (actually 3*24 hours)

```
cd source-of-the-site
code-maven recent --days 3
```

## Logging

You can add the `--debug` flag to get detailed logging of what's going on, the flag must come **before** the command:

```
code-maven --debug web
```

## GitHub pages

In order to setup a site on GitHub pages crate a file called `/.github/workflows/gh-pages.yml`  (the folder matters, the actual name of the file can be anything as long as the extension is `.yml` or `.yaml`.)

The content of the file:

{% include file="include/gh-pages.yml" %}


## Questions and Support

If you have encountered a bug, or if you would like to have a new feature, or just would like to ask a question, please open an [issue](https://github.com/szabgab/code-maven.rs/).

