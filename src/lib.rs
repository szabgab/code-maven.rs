use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;

use regex::Captures;
use regex::Regex;
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

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Author {
    pub name: String,
    pub nickname: String,
    pub picture: String,

    #[serde(default = "get_empty_string")]
    pub text: String,
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
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Link {
    from_title: String,
    from_path: String,
    to_title: String,
    to_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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

    pub published: bool,
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
            author: String::new(),
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
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

pub fn read_pages(config: &Config, path: &Path, root: &str) -> (Vec<Page>, Vec<PathBuf>) {
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
        let (page, paths) = match read_md_file(config, root, entry.path().to_str().unwrap()) {
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
    let links = collect_all_the_internal_links(&pages);
    //dbg!(&links);
    // for page in &pages {
    //     //dbg!(&page.content);
    //     let links = find_links(&page);
    //     dbg!(links);
    // }

    //let page = &pages[0];
    //let backlinks: Vec<Link> = links.into_iter().filter(|link| link.to_path == page.url_path).collect();
    //let backlinks  = links.into_iter().filter(|link| link.to_path == page.url_path).collect::<Vec<Link>>();
    pages = pages
        .into_iter()
        .map(|mut page| {
            // TODO can we limit the clone to the already filtered values?
            page.backlinks = links
                .clone()
                .into_iter()
                .filter(|link| link.to_path == format!("/{}", page.url_path))
                .collect::<Vec<Link>>();
            //log::info!("{:?}", page.backlinks);
            page
        })
        .collect();

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

    (pages, paths_to_copy)
}

pub fn read_md_file(
    config: &Config,
    root: &str,
    path: &str,
) -> Result<(Page, Vec<PathBuf>), String> {
    let mut page = Page::new();

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

    let (content, paths) = pre_process(config, root, &content);
    //page.backlinks = find_links(&content);

    let content = markdown2html(&content)
        .replace("<h1>", "<h1 class=\"title\">")
        .replace("<h2>", "<h2 class=\"title is-4\">")
        .replace("<h3>", "<h3 class=\"title is-5\">");

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

    Ok((page, paths))
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
    let re = Regex::new(r#"<a href="([^"]+)">([^<]+)</a>"#).unwrap();
    for capture in re.captures_iter(&page.content) {
        if capture[1].starts_with("http://") || capture[1].starts_with("https://") {
            continue;
        }
        links.push(Link {
            from_title: page.title.clone(),
            from_path: page.url_path.clone(),
            to_title: capture[2].to_string(),
            to_path: capture[1].to_string(),
        });
    }

    links
}

fn pre_process(config: &Config, root: &str, text: &str) -> (String, Vec<PathBuf>) {
    log::info!("pre_process");
    let re = Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").unwrap();
    let ext_to_language: HashMap<String, String> = read_languages();
    let mut paths: Vec<PathBuf> = vec![];

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
            paths.push(path.to_path_buf());
            caps[0].to_string() // .copy() // don't replace anything
        }
    });

    (result.to_string(), paths)
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
        content: "<p>Some Text.</p>\n<p>Some more text after an empty row.</p>\n<h2 class=\"title is-4\">A title with two hash-marks</h2>\n<p>More text <a href=\"/with_todo\">with TODO</a>.</p>\n".to_string(),
        // footer: <p><a href=\"https://github.com/szabgab/rust.code-maven.com/blob/main/pages/index.md\">source</a></p>
        //     Link {
        //         title: "with TODO".to_string(),
        //         path: "/with_todo".to_string()
        //     }
        // ],
        published: true,
        ..Page::default()
    };
    let expected = (expected_page, vec![]);
    assert_eq!(data, expected);
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
        published: true,
        ..Page::default()
    };
    let expected = (expected_page, vec![]);
    assert_eq!(data, expected);
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
        content: "<p><img src=\"examples/files/code_maven_490_490.jpg\" alt=\"a title\" /></p>\n"
            .to_string(),
        // footer: <p><a href=\"https://github.com/szabgab/rust.code-maven.com/blob/main/pages/img_with_title.md\">source</a></p>
        tags: vec!["img".to_string()],
        published: true,
        ..Page::default()
    };
    let expected = (
        expected_page,
        vec![PathBuf::from("examples/files/code_maven_490_490.jpg")],
    );
    assert_eq!(data, expected);
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
        content: "<ul>\n<li>An <a href=\"/with_todo\">internal link</a> and more text.</li>\n<li>An <a href=\"https://rust-digger.code-maven.com/\">external link</a> and more text.</li>\n</ul>\n".to_string(),
        //footer: "\n<p><a href=\"https://github.com/szabgab/rust.code-maven.com/blob/main/pages/links.md\">source</a></p>".to_strin(),
        //     Link {
        //         title: "internal link".to_string(),
        //         path: "/with_todo".to_string(),
        //     },
        // ],
        published: true,
        ..Page::default()
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
