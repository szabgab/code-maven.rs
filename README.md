# Code Maven implemented in Rust

* Start as a static site generator but in a way that it will be easy to convert it into a real web site

* Go over all the .md files and call the page generator and then save the files.
* Go over all the extra pages (e.g. /archive) and create the pages for those.


* Read .md files with a header that looks like this (we'll define the exact fields later)


## Installation

See [the web site](https://ssg.code-maven.com/)


## Setup a web site

See [the web site](https://ssg.code-maven.com/)


## Format

See [the web site](https://ssg.code-maven.com/)

## Local development

* Install Rust
* Clone https://github.com/szabgab/code-maven.rs
* cd code-maven.rs

Install [pre-commit](https://pre-commit.com/) and run `pre-commit install` to configure it for this repo.

* In the `demo` folder we have the files of a web site.
* In the `demo_site` folder we have the generated version of the demo site.
* Running the `tests.sh` will generated the demo site and compare it to the stored version.

Generate the demo pages:

```
cargo run --bin code-maven -- web --root test_cases/demo/ --outdir _site/
```

We can also explicitly say where the pages are, but so far we did not need that.

```
cargo run --bin code-maven -- web --root test_cases/demo/ --pages test_cases/demo/pages/ --outdir _site/
```

Assuming you have cloned https://github.com/szabgab/rust.code-maven.com/ next to this repository then you can also try:

```
cargo run --bin code-maven -- web --root ../rust.code-maven.com/ --outdir _site/
```


Install [rustatic](https://rustatic.code-maven.com/) and run the following to view the generated site

```
rustatic --path _site/ --indexfile index.html --nice --port 3000
```

## Sending emails

code-maven sendgrid --mail emails/content.md --tofile list-of-addresses.csv

## Logging / Debugging

code-maven --debug sendgrid


## Testing

```
cargo test
```

or for more verbose output use:

```
cargo t
```

Run the cargo test and also generate the demo site and the web site of SSG and compare then to saved version of the output.

```
./test.sh
```

Clones several public repositories of site and check if they can be generated? This is used in the GtHub Actions CI.

```
./check_sites.sh
```


