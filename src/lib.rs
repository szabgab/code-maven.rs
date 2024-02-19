use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;

use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use liquid_core::{
    Display_filter, Filter, FilterReflection, ParseFilter, Result, Runtime, Value, ValueView,
};

pub mod drafts;
pub mod new;
pub mod notifications;
pub mod recent;
pub mod todo;
pub mod web;

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "topath",
    description = "Convert a string to something we can use as a path in the URL",
    parsed(ToPathFilter)
)]
pub struct ToPath;

#[derive(Debug, Default, Display_filter)]
#[name = "topath"]
pub struct ToPathFilter;

impl Filter for ToPathFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
        let text = input.to_kstr();
        Ok(Value::scalar(topath(&text)))
    }
}

fn get_pages_path(root: &str, pages: &str) -> PathBuf {
    if pages.is_empty() {
        return PathBuf::from(root).join("pages");
    }
    PathBuf::from(pages)
}

pub fn topath(text: &str) -> String {
    match text {
        "!" => "exclamation-mark".to_string(),
        "\"" => "double-quote".to_string(),
        "#" => "number-sign".to_string(),
        "$" => "dollar".to_string(),
        "%" => "percent-sign".to_string(),
        "&" => "ampersand".to_string(),
        "'" => "single-quote".to_string(),
        "(" => "open-parenthesis".to_string(),
        ")" => "close-parenthesis".to_string(),
        "*" => "asterisk".to_string(),
        "+" => "plus".to_string(),
        "," => "comma".to_string(),
        "-" => "hyphen-minus".to_string(),
        "." => "full-stop".to_string(),
        "/" => "forward-slash".to_string(),
        ":" => "colon".to_string(),
        ";" => "semi-colon".to_string(),
        "<" => "less-than".to_string(),
        "=" => "equals".to_string(),
        ">" => "greater-than".to_string(),
        "?" => "question-mark".to_string(),
        "@" => "at-sign".to_string(),
        "[" => "opening-bracket".to_string(),
        "\\" => "back-slash".to_string(),
        "]" => "closing-bracket".to_string(),
        "^" => "caret".to_string(),
        //        "_" => "underscore".to_string(),
        "`" => "backtick".to_string(), // grave accent
        _ => text.replace(' ', "_").to_lowercase(),
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigNavbarLink {
    pub path: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigNavbar {
    pub start: Vec<ConfigNavbarLink>,
    pub end: Vec<ConfigNavbarLink>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFrom {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigArchive {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigTag {
    pub description: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Author {
    pub name: String,
    pub nickname: String,
    pub picture: String,

    #[serde(default = "get_empty_string")]
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub url: String,
    pub repo: String,
    pub branch: String,
    pub link_to_source: bool,
    pub tags: ConfigTag,
    pub archive: ConfigArchive,
    pub from: Option<ConfigFrom>,

    pub authors: Vec<Author>,

    #[serde(default = "get_empty_string")]
    pub footer: String,

    #[serde(default = "get_empty_string")]
    pub site_name: String,

    #[serde(default = "get_empty_string")]
    pub author_name: String,

    pub navbar: ConfigNavbar,

    #[serde(default = "get_empty_string")]
    pub google_analytics: String,

    #[serde(default = "get_false")]
    pub show_related: bool,

    #[serde(default = "get_empty_string")]
    pub related_pages_title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Link {
    from_title: String,
    from_path: String,
    to_title: String,
    to_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub title: String,
    pub timestamp: String,

    #[serde(default = "get_empty_string")]
    pub url_path: String,

    #[serde(default = "get_empty_string")]
    pub filename: String,

    #[serde(default = "get_empty_string")]
    pub description: String,

    #[serde(default = "get_empty_vector")]
    pub todo: Vec<String>,

    #[serde(default = "get_empty_vector")]
    pub tags: Vec<String>,

    #[serde(default = "get_empty_string")]
    pub content: String,

    #[serde(default = "get_empty_links")]
    pub backlinks: Vec<Link>,

    #[serde(default = "get_empty_string")]
    pub author: String,

    pub redirect: Option<String>,

    pub published: bool,

    #[serde(default = "get_true")]
    pub show_related: bool,
}

impl Page {
    pub fn new() -> Page {
        Page {
            title: String::new(),
            timestamp: String::new(),
            description: String::new(),
            url_path: String::new(),
            filename: String::new(),
            content: String::new(),
            todo: vec![],
            tags: vec![],
            backlinks: vec![],
            published: false,
            redirect: None,
            show_related: true,
            author: String::new(),
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

fn get_true() -> bool {
    true
}

fn get_false() -> bool {
    false
}

fn get_empty_links() -> Vec<Link> {
    vec![]
}

fn get_empty_vector() -> Vec<String> {
    vec![]
}

fn get_empty_string() -> String {
    String::new()
}

pub fn markdown_pages(pages: Vec<Page>) -> Vec<Page> {
    pages
        .into_iter()
        .map(|mut page| {
            page.content = markdown2html(&page.content)
                .replace("<h1>", "<h1 class=\"title\">")
                .replace("<h2>", "<h2 class=\"title is-4\">")
                .replace("<h3>", "<h3 class=\"title is-5\">");
            page
        })
        .collect()
}

fn process_liquid_tags_youtube(text: &str) -> String {
    let re = Regex::new(r#"\{%\s+youtube="([^"]+)"\s+%\}"#).unwrap();
    re.replace_all(text, r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/$1" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe>"#).to_string()
}

fn process_liquid_tags_for_text(text: &str, all_pages: &[Page]) -> String {
    let re = Regex::new(r#"\{%\s+latest\s+limit=(\d+)\s+(?:tag=(\S+)\s+)?%\}"#).unwrap();
    re.replace_all(text, |caps: &Captures| {
        let mut count = 0;
        let limit = caps[1].parse::<usize>().unwrap();
        let tag = caps.get(2);

        let mut html = String::new();
        #[allow(clippy::needless_range_loop)]
        for ix in 1..all_pages.len() {
            if tag.is_some() {
                let tag_text = &tag.unwrap().as_str().to_string();
                if !all_pages[ix].tags.contains(tag_text) {
                    continue;
                }
            }
            html += format!("* [{}](/{})", all_pages[ix].title, all_pages[ix].url_path).as_str();
            html += "\n";
            count += 1;
            if 0 < limit && limit <= count {
                break;
            }
        }

        html
    })
    .to_string()
}

pub fn get_files_to_copy(pages: &Vec<Page>) -> Vec<PathBuf> {
    let mut paths_to_copy: Vec<PathBuf> = vec![];
    let mut in_code = false;
    let re = Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").unwrap();
    let ext_images: Vec<&str> = vec!["png", "jpg", "jpeg", "gif"];

    for page in pages {
        for row in page.content.split('\n') {
            if row.starts_with("```") {
                in_code = !in_code;
                continue;
            }
            if !in_code {
                if let Some(value) = re.captures(row) {
                    let path = Path::new(&value[1]);
                    // TODO: we don't need to copy external images
                    if ext_images.contains(&path.extension().unwrap().to_str().unwrap()) {
                        paths_to_copy.push(path.to_path_buf().clone());
                    } else {
                        log::error!(
                            "Unhandled extension for file to be copied {:?}",
                            path.file_name().unwrap()
                        );
                        std::process::exit(1);
                    }
                };
            }
        }
    }

    paths_to_copy
}

pub fn process_liquid_tags(config: &Config, root: &str, pages: Vec<Page>) -> Vec<Page> {
    let all_pages = pages.clone();

    let pages = pages
        .into_iter()
        .map(|mut page| {
            let mut in_code = false;
            page.content = page
                .content
                .split('\n')
                .map(|row| {
                    if row.starts_with("```") {
                        in_code = !in_code;
                    }
                    if in_code {
                        row.to_owned()
                    } else {
                        let row = process_liquid_tags_for_text(row, &all_pages);
                        let row = process_liquid_tags_youtube(&row);
                        process_liquid_include(config, root, &row)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
            page
        })
        .collect::<Vec<Page>>();
    pages
}

fn collect_backlinks(pages: Vec<Page>) -> Vec<Page> {
    let links = collect_all_the_internal_links(&pages);

    pages
        .into_iter()
        .map(|mut page| {
            // TODO can we limit the clone to the already filtered values?
            let mut backlinks = links
                .clone()
                .into_iter()
                .filter(|link| link.to_path == format!("/{}", page.url_path))
                .collect::<Vec<Link>>();
            backlinks.sort_by(|a, b| b.from_title.cmp(&a.from_title));
            page.backlinks = backlinks;
            //log::info!("{:?}", page.backlinks);
            page
        })
        .collect()
}

pub fn read_pages(config: &Config, path: &Path, root: &str) -> Vec<Page> {
    log::info!("read_page from path '{:?}'", path);
    let mut pages: Vec<Page> = vec![];

    let dir = match path.read_dir() {
        Ok(dir) => dir,
        Err(err) => {
            log::error!("read_dir of '{path:?}' call failed: {err}");
            std::process::exit(1);
        }
    };

    for entry in dir.flatten() {
        log::info!("path: {:?}", entry.path());
        if entry.path().extension().unwrap() != "md" {
            log::info!("Skipping non-md file '{:?}'", entry.path().to_str());
            continue;
        }
        // println!("{:?}", entry.file_name());
        let page = match read_md_file(config, root, entry.path().to_str().unwrap()) {
            Ok(res) => res,
            Err(err) => {
                log::error!("{}", err);
                std::process::exit(1);
            }
        };
        log::debug!("page: {:?}", &page);

        pages.push(page);
    }

    match check_unique_dates(&pages) {
        Ok(()) => {}
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        }
    };
    pages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let archive = Page {
        url_path: String::from("archive"),
        published: true,
        timestamp: if pages.is_empty() {
            let now: DateTime<Utc> = Utc::now();
            now.format("%Y-%m-%dT%H:%M:%S").to_string()
        } else {
            pages[0].timestamp.clone()
        },
        ..Page::default()
    };

    pages.insert(0, archive);

    pages
}

fn check_unique_dates(pages: &Vec<Page>) -> Result<(), String> {
    let mut uniq = std::collections::HashSet::new();
    for page in pages {
        if uniq.contains(&page.timestamp) {
            return Err(format!("duplicate timestamp '{}'", page.timestamp));
        }
        uniq.insert(page.timestamp.clone());
    }

    Ok(())
}

pub fn read_md_file(_config: &Config, _root: &str, path: &str) -> Result<Page, String> {
    let mut page = Page::new();
    log::info!("read_md_file '{path}'");

    let mut content = String::new();

    if !std::path::Path::new(path).exists() {
        return Err(format!("File '{path}' not found"));
    }

    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut in_front_matter = false;
            let mut front_matter = String::new();
            for line in reader.lines() {
                let line = line.unwrap();
                log::debug!("line '{}'", line);
                if in_front_matter {
                    if line == "---" {
                        in_front_matter = false;
                        log::info!("front_matter: '{}'", &front_matter);
                        match serde_yaml::from_str(&front_matter) {
                            Ok(val) => page = val,
                            Err(err) => {
                                return Err(format!("YAML parsing error in '{path}' {err}"))
                            }
                        };
                        continue;
                    }
                    //dbg!(&line);
                    front_matter += &line;
                    front_matter += "\n";
                    continue;
                }
                if line == "---" && front_matter.is_empty() {
                    in_front_matter = true;
                    continue;
                }

                content += &line;
                content += "\n";
            }
        }
        Err(error) => {
            log::error!("Error opening file {path}: {error}");
        }
    }

    if page.tags.iter().any(String::is_empty) {
        return Err(format!("There is an empty tag in {path}"));
    }

    let mut p = PathBuf::from(path);
    page.filename = p.file_name().unwrap().to_str().unwrap().to_string();
    p.set_extension("");
    page.url_path = p.file_name().unwrap().to_str().unwrap().to_string();

    if page.url_path == "index" {
        page.url_path = String::new();
    }

    page.content = content;

    if page.title.is_empty() {
        return Err(format!("Missing title in '{path}'"));
    }
    match chrono::NaiveDateTime::parse_from_str(&page.timestamp, "%Y-%m-%dT%H:%M:%S") {
        Ok(_) => {
            let _x = 1;
        }
        Err(err) => {
            return Err(format!(
                "Invalid date '{}' in {}: {}",
                page.timestamp, path, err
            ));
        }
    }

    Ok(page)
}

fn markdown2html(content: &str) -> String {
    markdown::to_html_with_options(
        content,
        &markdown::Options {
            compile: markdown::CompileOptions {
                allow_dangerous_html: true,
                //allow_dangerous_protocol: true,
                ..markdown::CompileOptions::default()
            },
            ..markdown::Options::gfm()
        },
    )
    .unwrap()
}

fn collect_all_the_internal_links(pages: &[Page]) -> Vec<Link> {
    pages
        .iter()
        .map(find_links)
        .collect::<Vec<Vec<Link>>>()
        .concat()
}

fn find_links(page: &Page) -> Vec<Link> {
    let mut links: Vec<Link> = vec![];

    // TODO include the internal links that have the site URL as well.
    let re = Regex::new(r#"\[([^]]+)\]\(([^)]+)\)"#).unwrap();
    for capture in re.captures_iter(&page.content) {
        if capture[2].starts_with("http://") || capture[2].starts_with("https://") {
            continue;
        }
        links.push(Link {
            from_title: page.title.clone(),
            from_path: page.url_path.clone(),
            to_title: capture[1].to_string(),
            to_path: capture[2].to_string(),
        });
    }

    links
}

fn process_liquid_include(config: &Config, root: &str, text: &str) -> String {
    log::info!("process_liquid_include for {text}");
    let ext_to_language: HashMap<String, String> = read_languages();

    let re = Regex::new(r#"\{%\s+include\s+file="([^"]+)"\s+%\}"#).unwrap();
    let result = re.replace_all(text, |caps: &Captures| {
        let path = Path::new(&caps[1]);
        let include_path = Path::new(root).join(path);
        log::debug!("path '{:?}'", path);
        // TODO remove the hard coded mapping of .gitignore
        // TODO properly handle files that do not have an extension
        if path.file_name().unwrap().to_str().unwrap() == ".gitignore" {
            include_file(config, include_path, path, "gitignore")
        } else if ext_to_language.contains_key(path.extension().unwrap().to_str().unwrap()) {
            let language = ext_to_language[path.extension().unwrap().to_str().unwrap()].as_str();
            include_file(config, include_path, path, language)
        } else {
            log::error!("Unhandled include statement for row {text}");
            std::process::exit(1);
        }
    });

    result.to_string()
}

fn copy_files(root: &str, outdir: &str, paths: &Vec<PathBuf>) {
    for path in paths {
        let include_path = Path::new(root).join(path);
        let output_path = Path::new(outdir).join(path);
        log::info!("copy file from '{:?}' to '{:?}'", include_path, output_path);
        copy_file(&include_path, &output_path);
    }
}

fn copy_file(source_path: &Path, destination_path: &PathBuf) {
    log::info!(
        "copy_path: from {:?} to {:?}",
        source_path,
        destination_path
    );
    let destination_dir = destination_path.parent().unwrap();
    log::info!("dir: {:?}", destination_dir);
    if !source_path.exists() {
        log::error!("source_path: {:?} does not exists", source_path);
        return;
    }

    if !destination_dir.exists() {
        fs::create_dir_all(destination_dir).unwrap();
    }
    fs::copy(source_path, destination_path).unwrap();
}

fn include_file(config: &Config, include_path: PathBuf, path: &Path, language: &str) -> String {
    log::info!("include_path: {:?}", include_path);

    match std::fs::read_to_string(&include_path) {
        Ok(file_content) => {
            format!(
                "**[{}]({}/tree/{}/{})**\n```{}\n{}\n```\n",
                path.display(),
                &config.repo,
                &config.branch,
                path.display(),
                language,
                &file_content
            )
        }
        Err(err) => {
            log::info!("Failed to include file {:?}: {}", include_path, err);
            "MISSING".to_string()
        }
    }
}

fn read_languages() -> HashMap<String, String> {
    let text = include_str!("../data/languages.csv");

    let mut data = HashMap::new();
    for line in text.split('\n') {
        if line.is_empty() {
            continue;
        }
        let parts = line.split(',').collect::<Vec<&str>>();
        data.insert(parts[0].to_string(), parts[1].to_string());
    }

    data
}

pub fn read_config(root: &str) -> Result<Config, String> {
    let filepath = std::path::Path::new(root).join("config.yaml");
    log::info!("read_config {:?}", filepath);

    let mut config: Config = match std::fs::File::open(&filepath) {
        Ok(file) => match serde_yaml::from_reader(file) {
            Ok(data) => data,
            Err(err) => {
                return Err(format!("Invalid YAML format in {filepath:?}: {err}"));
            }
        },
        Err(error) => {
            return Err(format!("Error opening file {filepath:?}: {error}"));
        }
    };

    let nicknames = config
        .authors
        .iter()
        .map(|author| author.nickname.clone())
        .collect::<Vec<String>>();
    let mut uniq = std::collections::HashSet::new();
    for nickname in nicknames {
        if uniq.contains(&nickname) {
            return Err(format!(
                "nickname '{nickname}' appears twice in config.yaml"
            ));
        }
        uniq.insert(nickname);
    }

    config.authors = config
        .authors
        .into_iter()
        .map(|mut author| {
            let author_file = std::path::Path::new(root)
                .join("authors")
                .join(format!("{}.md", author.nickname));
            let content = std::fs::read_to_string(author_file).unwrap_or_default();
            author.text = markdown2html(&content);
            author
        })
        .collect::<Vec<Author>>();

    Ok(config)
}

pub fn filter_words(words: &[String]) -> Vec<String> {
    words
        .to_owned()
        .clone()
        .into_iter()
        .filter(|word| word.chars().all(|chr| chr.is_alphanumeric() || chr == ' '))
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_topath() {
        let cases = vec![("hello", "hello"), ("#", "number-sign")];

        for entry in cases {
            let text = "{{ text | topath}}";
            let globals = liquid::object!({
                "text": entry.0,
            });
            let template = liquid::ParserBuilder::with_stdlib()
                .filter(ToPath)
                .build()
                .unwrap()
                .parse(text)
                .unwrap();
            let output = template.render(&globals).unwrap();
            assert_eq!(output, entry.1.to_string());
        }
    }
}

#[test]
fn test_read_index() {
    let config = read_config("test_cases/demo").unwrap();
    let data = read_md_file(&config, "test_cases/demo", "test_cases/demo/pages/index.md").unwrap();
    dbg!(&data);
    let expected_page = Page {
        title: "Index page".to_string(),
        timestamp: "2015-10-11T12:30:01".to_string(),
        description: "The text for the search engines".to_string(),
        filename: "index.md".to_string(),
        content: "\nSome Text.\n\nSome more text after an empty row.\n\n## A title with two hash-marks\n\nMore text [with TODO](/with_todo).\n".to_string(),
        published: true,
        ..Page::default()
    };
    //let expected = (expected_page, vec![]);
    assert_eq!(data, expected_page);
}

#[test]
fn test_read_todo() {
    let config = read_config("test_cases/demo").unwrap();
    let data = read_md_file(
        &config,
        "test_cases/demo",
        "test_cases/demo/pages/with_todo.md",
    )
    .unwrap();
    dbg!(&data);
    let expected_page = Page {
        title: "Page with todos".to_string(),
        timestamp: "2023-10-11T12:30:01".to_string(),
        url_path: "with_todo".to_string(),
        filename: "with_todo.md".to_string(),
        content: "\nSome Content.\n\n{% include file=\"examples/hello_world.rs\" %}\n".to_string(),
        todo: vec![
            "Add another article extending on the topic".to_string(),
            "Add an article describing a prerequisite".to_string(),
        ],
        tags: vec!["println!".to_string(), "fn".to_string()],
        published: true,
        ..Page::default()
    };
    //let expected = (expected_page, vec![]);
    assert_eq!(data, expected_page);
}

#[test]
fn test_img_with_title() {
    let config = read_config("test_cases/demo").unwrap();
    let data = read_md_file(
        &config,
        "test_cases/demo",
        "test_cases/demo/pages/img_with_title.md",
    )
    .unwrap();
    dbg!(&data);
    let expected_page = Page {
        title: "Image with title".to_string(),
        timestamp: "2023-10-03T13:30:01".to_string(),
        url_path: "img_with_title".to_string(),
        filename: "img_with_title.md".to_string(),
        content: "\n\n![a title](examples/files/code_maven_490_490.jpg)\n\n".to_string(),
        tags: vec!["img".to_string()],
        published: true,
        ..Page::default()
    };
    // let expected = (
    //     expected_page,
    //     vec![PathBuf::from("examples/files/code_maven_490_490.jpg")],
    // );
    assert_eq!(data, expected_page);
}

#[test]
fn test_links() {
    let config = read_config("test_cases/demo").unwrap();
    let data = read_md_file(&config, "test_cases/demo", "test_cases/demo/pages/links.md").unwrap();
    dbg!(&data);
    let expected_page = Page {
        title: "Links".to_string(),
        timestamp: "2023-10-01T12:30:01".to_string(),
        url_path: "links".to_string(),
        filename: "links.md".to_string(),
        content: "\n* An [internal link](/with_todo) and more text.\n* An [external link](https://rust-digger.code-maven.com/) and more text.\n\n[sigils](/sigils) - another internal link to test the `show_related: false` in the front-matter of the sigils page\n[sub](/sub) - another internal link to test the `show_related: true` in the front-matter of the sigils page\n".to_string(),
        published: true,
        ..Page::default()
    };
    //let expected = (expected_page, vec![]);
    assert_eq!(data, expected_page);
}

#[test]
fn test_filter_words() {
    let original = vec![
        "hello".to_string(),
        "one 2 three".to_string(),
        "'".to_string(),
    ];
    let expected = vec!["hello".to_string(), "one 2 three".to_string()];

    let filtered = filter_words(&original);
    assert_eq!(filtered, expected);
}

#[test]
fn test_missing_md_file() {
    let config = read_config("test_cases/demo").unwrap();
    match read_md_file(
        &config,
        "test_cases/demo",
        "test_cases/demo/pages/no_such_file.md",
    ) {
        Ok(_) => assert!(false),
        Err(err) => assert_eq!(
            err,
            "File 'test_cases/demo/pages/no_such_file.md' not found".to_string()
        ),
    }
}

#[test]
fn test_missing_title() {
    let config = read_config("test_cases/demo").unwrap();
    match read_md_file(
        &config,
        "test_cases/demo",
        "test_cases/bad_pages/missing_front_matter.md",
    ) {
        Ok(_) => assert!(false),
        Err(err) => assert_eq!(
            err,
            "Missing title in 'test_cases/bad_pages/missing_front_matter.md'".to_string()
        ),
    }
}

#[test]
fn test_bad_timestamp() {
    let config = read_config("test_cases/demo").unwrap();
    match read_md_file(&config, "test_cases/demo", "test_cases/bad_pages/incorrect_timestamp.md") {
        Ok(_) => assert!(false),
        Err(err) => assert_eq!(
            err,
            "Invalid date '2015-02-30T12:30:01' in test_cases/bad_pages/incorrect_timestamp.md: input is out of range".to_string()
        ),
    }
}

#[test]
fn test_invalid_key() {
    //simple_logger::init_with_level(log::Level::Debug).unwrap();
    let config = read_config("test_cases/demo").unwrap();
    match read_md_file(&config, "test_cases/demo", "test_cases/bad_pages/invalid_key_in_front_matter.md") {
        Ok(_) => assert!(false),
        Err(err) => assert!(
            err.contains("YAML parsing error in 'test_cases/bad_pages/invalid_key_in_front_matter.md' unknown field `password`")
        ),
    }
}

#[allow(unused_macros)]
macro_rules! s(($result:expr) => ($result.to_string()));

#[test]
fn test_config_of_demo() {
    let config = read_config("test_cases/demo").unwrap();
    assert_eq!(config.url, "https://rust.code-maven.com");
    assert_eq!(
        config.repo,
        "https://github.com/szabgab/rust.code-maven.com"
    );
    assert_eq!(
        config.authors,
        vec![Author {
            name: "Gabor Szabo".to_string(),
            nickname: s!("szabgab"),
            picture: s!("szabgab.png"),
            text: s!(""),
        }]
    );
}

#[test]
fn test_config_with_author_files() {
    let config = read_config("test_cases/config_with_authors/").unwrap();
    assert_eq!(config.url, "https://rust.code-maven.com");
    assert_eq!(
        config.repo,
        "https://github.com/szabgab/rust.code-maven.com"
    );
    assert_eq!(
        config.authors,
        vec![Author {
            name: "Gabor Szabo".to_string(),
            nickname: s!("szabgab"),
            picture: s!("szabgab.png"),
            text: s!(
                r#"<p><a href="https://szabgab.com/">Gabor Szabo</a>, the author of the Rust Maven web site
teaches Rust, Python, git, CI, and testing.</p>
"#
            ),
        }]
    );
}

#[test]
fn test_config_with_duplicate_author() {
    let error = read_config("test_cases/same_nickname_twice/")
        .err()
        .unwrap();
    assert_eq!(error, "nickname 'foobar' appears twice in config.yaml")
}

#[test]
fn test_config_with_invalid_field() {
    let error = read_config("test_cases/invalid_field_in_config/")
        .err()
        .unwrap();
    assert!(error.contains(r#"Invalid YAML format in "test_cases/invalid_field_in_config/config.yaml": unknown field `password`,"#))
}
