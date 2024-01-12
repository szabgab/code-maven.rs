use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;

use regex::Captures;
use regex::Regex;
use serde::{Deserialize, Serialize};

use liquid_core::{
    Display_filter, Filter, FilterReflection, ParseFilter, Result, Runtime, Value, ValueView,
};

pub mod cm_sendgrid;
pub mod drafts;
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
pub struct ConfigNavbarLink {
    pub path: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigNavbar {
    pub start: Vec<ConfigNavbarLink>,
    pub end: Vec<ConfigNavbarLink>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFrom {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigArchive {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigTag {
    pub description: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub url: String,
    pub repo: String,
    pub branch: String,
    pub link_to_source: bool,
    pub tags: ConfigTag,
    pub archive: ConfigArchive,
    pub from: Option<ConfigFrom>,

    #[serde(default = "get_empty_string")]
    pub footer: String,

    #[serde(default = "get_empty_string")]
    pub site_name: String,

    pub navbar: ConfigNavbar,

    #[serde(default = "get_empty_string")]
    pub google_analytics: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Link {
    title: String,
    path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Page {
    pub title: String,
    pub timestamp: String,

    #[serde(default = "get_empty_string")]
    pub filename: String,

    #[serde(default = "get_empty_string")]
    pub description: String,

    #[serde(default = "get_empty_vector")]
    todo: Vec<String>,

    #[serde(default = "get_empty_vector")]
    pub tags: Vec<String>,

    #[serde(default = "get_empty_string")]
    pub content: String,

    #[serde(default = "get_empty_links")]
    backlinks: Vec<Link>,

    pub published: bool,
}

impl Page {
    pub fn new() -> Page {
        Page {
            title: "".to_string(),
            timestamp: "".to_string(),
            description: "".to_string(),
            filename: "".to_string(),
            content: "".to_string(),
            todo: vec![],
            tags: vec![],
            backlinks: vec![],
            published: true,
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

fn get_empty_links() -> Vec<Link> {
    vec![]
}

fn get_empty_vector() -> Vec<String> {
    vec![]
}

fn get_empty_string() -> String {
    "".to_string()
}

pub fn read_pages(
    config: &Config,
    path: &Path,
    root: &str,
    outdir: &str,
) -> (Vec<Page>, Vec<PathBuf>) {
    log::info!("read_page from path '{:?}'", path);
    let mut pages: Vec<Page> = vec![];
    let mut paths_to_copy: Vec<PathBuf> = vec![];

    for entry in path.read_dir().expect("read_dir call failed").flatten() {
        log::info!("path: {:?}", entry.path());
        if entry.path().extension().unwrap() != "md" {
            log::info!("Skipping non-md file '{:?}'", entry.path().to_str());
            continue;
        }
        // println!("{:?}", entry.file_name());
        let (page, paths) = match read_md_file(config, root, entry.path().to_str().unwrap(), outdir)
        {
            Ok(res) => res,
            Err(err) => {
                log::error!("{}", err);
                std::process::exit(1);
            }
        };
        log::debug!("page: {:?}", &page);
        paths_to_copy.extend(paths);
        pages.push(page);
    }

    pages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let mut archive = Page::new();
    archive.filename = "archive".to_string();
    if pages.is_empty() {
        let now: DateTime<Utc> = Utc::now();
        archive.timestamp = now.format("%Y-%m-%dT%H:%M:%S").to_string();
    } else {
        archive.timestamp = pages[0].timestamp.clone();
    }

    pages.insert(0, archive);

    (pages, paths_to_copy)
}

pub fn read_md_file(
    config: &Config,
    root: &str,
    path: &str,
    outdir: &str,
) -> Result<(Page, Vec<PathBuf>), String> {
    let mut page: Page = Page::new();

    let mut content = "".to_string();

    if !std::path::Path::new(path).exists() {
        return Err(format!("File '{}' not found", path));
    }

    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut in_front_matter = false;
            let mut front_matter = "".to_string();
            for line in reader.lines() {
                let line = line.unwrap();
                log::debug!("line '{}'", line);
                if in_front_matter {
                    if line == "---" {
                        in_front_matter = false;
                        log::info!("front_matter: '{}'", &front_matter);
                        page = serde_yaml::from_str(&front_matter).unwrap_or_else(|err| {
                            log::error!("YAML parsing error in '{}' {}", path, err);
                            std::process::exit(1);
                        });
                        continue;
                    }
                    //dbg!(&line);
                    front_matter += &line;
                    front_matter += "\n";
                    continue;
                }
                if line == "---" {
                    in_front_matter = true;
                    continue;
                }

                content += &line;
                content += "\n";
            }
        }
        Err(error) => {
            println!("Error opening file {}: {}", path, error);
        }
    }

    if page.tags.iter().any(std::string::String::is_empty) {
        return Err(format!("There is an empty tag in {}", path));
    }

    let mut p = PathBuf::from(path);
    p.set_extension("");
    page.filename = p.file_name().unwrap().to_str().unwrap().to_string();

    let (content, paths) = pre_process(config, root, outdir, &content);
    page.backlinks = find_links(&content);

    let content = markdown::to_html_with_options(
        &content,
        &markdown::Options {
            compile: markdown::CompileOptions {
                allow_dangerous_html: true,
                //allow_dangerous_protocol: true,
                ..markdown::CompileOptions::default()
            },
            ..markdown::Options::gfm()
        },
    )
    .unwrap();
    //println!("{}", content);
    let content = content.replace("<h1>", "<h1 class=\"title\">");
    let content = content.replace("<h2>", "<h2 class=\"title is-4\">");
    let content = content.replace("<h3>", "<h3 class=\"title is-5\">");

    page.content = content;

    if page.title.is_empty() {
        return Err(format!("Missing title in '{}'", path));
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

    Ok((page, paths))
}

fn find_links(text: &str) -> Vec<Link> {
    let mut links: Vec<Link> = vec![];

    let re = Regex::new(r"[^!]\[([^\]]+)\]\(([^)]+)\)").unwrap();
    for capture in re.captures_iter(text) {
        if capture[2].starts_with("http://") || capture[2].starts_with("https://") {
            continue;
        }
        links.push(Link {
            title: capture[1].to_string(),
            path: capture[2].to_string(),
        });
    }

    links
}

fn pre_process(config: &Config, root: &str, outdir: &str, text: &str) -> (String, Vec<PathBuf>) {
    log::info!("pre_process");
    let re = Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").unwrap();
    let ext_to_language: HashMap<String, String> = read_languages();
    let paths: Vec<PathBuf> = vec![];

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
            // TODO: we don't need to copy external images
            let output_path = Path::new(outdir).join(path);
            log::info!("copy file from '{:?}' to '{:?}'", include_path, output_path);
            copy_file(&include_path, &output_path);
            caps[0].to_string() // .copy() // don't replace anything
        }
    });

    (result.to_string(), paths)
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

    if include_path.exists() {
        match File::open(include_path) {
            Ok(mut file) => {
                let mut content = "".to_string();
                content += &format!(
                    "**[{}]({}/tree/{}/{})**\n",
                    path.display(),
                    &config.repo,
                    &config.branch,
                    path.display()
                );
                content += "```";
                content += language;
                content += "\n";
                file.read_to_string(&mut content).unwrap();
                content += "\n";
                content += "```\n";
                content
            }
            Err(_error) => {
                //println!("Error opening file {}: {}", include_path.display(), error);
                "FAILED".to_string()
            }
        }
    } else {
        "MISSING".to_string()
    }
}

fn read_languages() -> HashMap<String, String> {
    let text = include_str!("../data/languages.csv");

    let mut data = HashMap::new();
    for line in text.split('\n') {
        if line.is_empty() {
            continue;
        }
        let parts = line.split(',');
        let parts: Vec<&str> = parts.collect();
        data.insert(parts[0].to_string(), parts[1].to_string());
    }

    data
}

pub fn read_config(root: &str) -> Config {
    let filepath = std::path::Path::new(root).join("config.yaml");
    log::info!("read_config {:?}", filepath);

    let config: Config = match std::fs::File::open(&filepath) {
        Ok(file) => match serde_yaml::from_reader(file) {
            Ok(data) => data,
            Err(err) => {
                log::error!("Invalid YAML format in {:?}: {}", filepath, err);
                std::process::exit(1);
            }
        },
        Err(error) => {
            log::error!("Error opening file {:?}: {}", filepath, error);
            std::process::exit(1);
        }
    };

    config
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
    let config = read_config("demo");
    let data = read_md_file(&config, "demo", "demo/pages/index.md", "temp").unwrap();
    dbg!(&data);
    let expected_page = Page {
        title: "Index page".to_string(),
        timestamp: "2015-10-11T12:30:01".to_string(),
        description: "The text for the search engines".to_string(),
        filename: "index".to_string(),
        content: "<p>Some Text.</p>\n<p>Some more text after an empty row.</p>\n<h2 class=\"title is-4\">A title with two hash-marks</h2>\n<p>More text <a href=\"/with_todo\">with TODO</a>.</p>\n".to_string(),
        // footer: <p><a href=\"https://github.com/szabgab/rust.code-maven.com/blob/main/pages/index.md\">source</a></p>
        todo: vec![],
        tags: vec![],
        backlinks: vec![
            Link {
                title: "with TODO".to_string(),
                path: "/with_todo".to_string()
            }
        ],
        published: true,
    };
    let expected = (expected_page, vec![]);
    assert_eq!(data, expected);
}

#[test]
fn test_read_todo() {
    let config = read_config("demo");
    let data = read_md_file(&config, "demo", "demo/pages/with_todo.md", "temp").unwrap();
    dbg!(&data);
    let expected_page = Page {
        title: "Page with todos".to_string(),
        timestamp: "2023-10-11T12:30:01".to_string(),
        description: "".to_string(),
        filename: "with_todo".to_string(),
        content: "<p>Some Content.</p>\n<p><strong><a href=\"https://github.com/szabgab/rust.code-maven.com/tree/main/examples/hello_world.rs\">examples/hello_world.rs</a></strong></p>\n<pre><code class=\"language-rust\">fn main() {\n    println!(&quot;Hello World!&quot;);\n}\n\n</code></pre>\n".to_string(),
        // footer <p><a href=\"https://github.com/szabgab/rust.code-maven.com/blob/main/pages/with_todo.md\">source</a></p>
        todo: vec![
            "Add another article extending on the topic".to_string(),
            "Add an article describing a prerequisite".to_string(),
        ],
        tags: vec![
            "println!".to_string(),
            "fn".to_string(),
        ],
        backlinks: vec![],
        published: true,
    };
    let expected = (expected_page, vec![]);
    assert_eq!(data, expected);
}

#[test]
fn test_img_with_title() {
    let config = read_config("demo");
    let data = read_md_file(&config, "demo", "demo/pages/img_with_title.md", "temp").unwrap();
    dbg!(&data);
    let expected_page = Page {
        title: "Image with title".to_string(),
        timestamp: "2023-10-03T13:30:01".to_string(),
        description: "".to_string(),
        filename: "img_with_title".to_string(),
        content: "<p><img src=\"examples/files/code_maven_490_490.jpg\" alt=\"a title\" /></p>\n"
            .to_string(),
        // footer: <p><a href=\"https://github.com/szabgab/rust.code-maven.com/blob/main/pages/img_with_title.md\">source</a></p>
        todo: vec![],
        tags: vec!["img".to_string()],
        backlinks: vec![],
        published: true,
    };
    let expected = (expected_page, vec![]);
    assert_eq!(data, expected);
}

#[test]
fn test_links() {
    let config = read_config("demo");
    let data = read_md_file(&config, "demo", "demo/pages/links.md", "temp").unwrap();
    dbg!(&data);
    let expected_page = Page {
        title: "Links".to_string(),
        timestamp: "2023-10-01T12:30:01".to_string(),
        description: "".to_string(),
        filename: "links".to_string(),
        content: "<ul>\n<li>An <a href=\"/with_todo\">internal link</a> and more text.</li>\n<li>An <a href=\"https://rust-digger.code-maven.com/\">external link</a> and more text.</li>\n</ul>\n".to_string(),
        //footer: "\n<p><a href=\"https://github.com/szabgab/rust.code-maven.com/blob/main/pages/links.md\">source</a></p>".to_strin(),
        todo: vec![
        ],
        tags: vec![],
        backlinks: vec![
            Link {
                title: "internal link".to_string(),
                path: "/with_todo".to_string(),
            },
        ],
        published: true,
    };
    let expected = (expected_page, vec![]);
    assert_eq!(data, expected);
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
    let config = read_config("demo");
    match read_md_file(&config, "demo", "demo/pages/no_such_file.md", "temp") {
        Ok(_) => assert!(false),
        Err(err) => assert_eq!(
            err,
            "File 'demo/pages/no_such_file.md' not found".to_string()
        ),
    }
}

#[test]
fn test_missing_title() {
    let config = read_config("demo");
    match read_md_file(&config, "demo", "demo/bad/missing_front_matter.md", "temp") {
        Ok(_) => assert!(false),
        Err(err) => assert_eq!(
            err,
            "Missing title in 'demo/bad/missing_front_matter.md'".to_string()
        ),
    }
}

#[test]
fn test_bad_timestamp() {
    let config = read_config("demo");
    match read_md_file(&config, "demo", "demo/bad/incorrect_timestamp.md", "temp") {
        Ok(_) => assert!(false),
        Err(err) => assert_eq!(
            err,
            "Invalid date '2015-02-30T12:30:01' in demo/bad/incorrect_timestamp.md: input is out of range".to_string()
        ),
    }
}
