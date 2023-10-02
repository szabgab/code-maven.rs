use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use clap::Parser;
use regex::Captures;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

type Tags = HashMap<String, i32>;

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    #[arg(long)]
    pages: String,

    #[arg(long)]
    root: String,

    #[arg(long)]
    outdir: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct Page {
    title: String,
    timestamp: String,

    #[serde(default = "get_empty_string")]
    filename: String,

    #[serde(default = "get_empty_string")]
    description: String,

    #[serde(default = "get_empty_vector")]
    todo: Vec<String>,

    #[serde(default = "get_empty_vector")]
    tags: Vec<String>,

    #[serde(default = "get_empty_string")]
    content: String,
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
        }
    }
}

fn get_empty_vector() -> Vec<String> {
    vec![]
}

fn get_empty_string() -> String {
    "".to_string()
}

fn main() {
    let args = Cli::parse();
    //println!("{:?}", &args);
    simple_logger::init_with_level(log::Level::Info).unwrap();
    log::info!("Generate pages");

    if !Path::new(&args.outdir).exists() {
        fs::create_dir(&args.outdir).unwrap();
    }
    let tags_dir = Path::new(&args.outdir).join("tags");
    if !Path::new(&tags_dir).exists() {
        fs::create_dir(tags_dir).unwrap();
    }
    let url = "https://rust.code-maven.com";
    let pages = read_pages(&args.pages, &args.root, &args.outdir);
    let tags: Tags = collect_tags(&pages);
    render_pages(&pages, &args.outdir);
    render_tag_pages(&pages, &tags, &args.outdir);
    render_sitemap(&pages, &format!("{}/sitemap.xml", &args.outdir), url);
    render_archive(pages, &format!("{}/archive.html", &args.outdir));
    render_robots_txt(&format!("{}/robots.txt", &args.outdir), url);
}

fn collect_tags(pages: &Vec<Page>) -> Tags {
    let mut tags: Tags = HashMap::new();
    for page in pages {
        for tag in &page.tags {
            tags.insert(tag.to_lowercase(), 1);
        }
    }
    tags
}

fn render_robots_txt(path: &str, url: &str) {
    let text = format!(
        "Sitemap: {}/sitemap.xml\nSitemap: {}/slides/sitemap.xml\n\nUser-agent: *\n",
        url, url
    );

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", text).unwrap();
}

fn render_sitemap(pages: &Vec<Page>, path: &str, url: &str) {
    log::info!("render sitemap");
    let template_filename = String::from("templates/sitemap.xml");
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse_file(template_filename)
        .unwrap();

    let globals = liquid::object!({
        "title": "Archive".to_string(),
        "pages": &pages,
        "url": url,
    });
    let output = template.render(&globals).unwrap();

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();
}

fn render_archive(pages: Vec<Page>, path: &str) {
    log::info!("render archive");

    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let filtered_pages: Vec<Page> = pages
        .into_iter()
        .filter(|page| page.filename != "index" && page.filename != "archive")
        .collect();
    let template_filename = String::from("templates/archive.html");
    let template = liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
        .parse_file(template_filename)
        .unwrap();

    let globals = liquid::object!({
        "title": "Archive".to_string(),
        "description": "List of all the articles about the Rust programming language".to_string(),
        "pages": &filtered_pages,
    });
    let output = template.render(&globals).unwrap();

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();
}

fn render_tag_pages(pages: &Vec<Page>, tags: &Tags, outdir: &str) {
    log::info!("render_tag_pages");
    for tag in tags.keys() {
        let mut pages_with_tag: Vec<Page> = vec![];
        for page in pages {
            for xtag in &page.tags {
                if &xtag.to_lowercase() == tag {
                    pages_with_tag.push(page.clone());
                }
            }
        }
        let mut path = Path::new(outdir).join("tags").join(tag);
        path.set_extension("html");
        log::info!("render_tag {}", tag);

        let partials = match load_templates() {
            Ok(partials) => partials,
            Err(error) => panic!("Error loading templates {}", error),
        };

        let template_filename = String::from("templates/tag.html");
        let template = liquid::ParserBuilder::with_stdlib()
            .partials(partials)
            .build()
            .unwrap()
            .parse_file(&template_filename)
            .unwrap();

        let globals = liquid::object!({
            "title": format!("Articles tagged with '{}'", tag),
            "description": format!("Articles about Rust tagged with '{}'", tag),
            "pages": pages_with_tag,
        });
        let output = template.render(&globals).unwrap();

        log::info!("saving file at {:?}", path);
        let mut file = File::create(path).unwrap();
        writeln!(&mut file, "{}", output).unwrap();
    }
}

fn render_pages(pages: &Vec<Page>, outdir: &str) {
    for page in pages {
        if page.filename == "archive" {
            continue;
        }
        let mut outfile = PathBuf::from(&page.filename);
        outfile.set_extension("html");
        render(page, &format!("{}/{}", outdir, outfile.display()));
    }
}

fn read_pages(pages_path: &str, root: &str, outdir: &str) -> Vec<Page> {
    let mut pages: Vec<Page> = vec![];
    let path = Path::new(pages_path);
    for entry in path.read_dir().expect("read_dir call failed").flatten() {
        log::info!("path: {:?}", entry.path());
        if entry.path().extension().unwrap() != "md" {
            log::info!("Skipping non-md file '{:?}'", entry.path().to_str());
            continue;
        }
        // println!("{:?}", entry.file_name());
        let page = read_md_file(root, entry.path().to_str().unwrap(), outdir);
        log::info!("{:?}", &page);
        pages.push(page);
    }

    let mut archive = Page::new();
    archive.filename = "archive".to_string();
    let now: DateTime<Utc> = Utc::now();
    archive.timestamp = now.format("%Y-%m-%dT%H::%M::%S").to_string();
    pages.push(archive);

    pages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    pages
}

pub fn load_templates() -> Result<Partials, Box<dyn Error>> {
    // log::info!("load_templates");

    let mut partials = Partials::empty();
    let filename = "templates/incl/header.html";
    partials.add(filename, read_file(filename));
    let filename = "templates/incl/footer.html";
    partials.add(filename, read_file(filename));
    let filename = "templates/incl/navigation.html";
    partials.add(filename, read_file(filename));
    Ok(partials)
}

pub fn read_file(filename: &str) -> String {
    let mut content = String::new();
    match File::open(filename) {
        Ok(mut file) => {
            file.read_to_string(&mut content).unwrap();
        }
        Err(error) => {
            println!("Error opening file {}: {}", filename, error);
        }
    }
    content
}

fn render(page: &Page, path: &str) {
    log::info!("render path {}", path);

    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let template_filename = String::from("templates/page.html");
    let template = liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
        .parse_file(template_filename)
        .unwrap();

    let globals = liquid::object!({
        "title": page.title,
        "description": page.description,
        "content": page.content,
        "page": page,
    });
    let output = template.render(&globals).unwrap();

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();
}

fn read_md_file(root: &str, path: &str, outdir: &str) -> Page {
    let mut page: Page = Page::new();

    let mut content = "".to_string();

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
                        log::info!("'{}'", &front_matter);
                        page = serde_yaml::from_str(&front_matter).expect("YAML parsing error");
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

    let mut p = PathBuf::from(path);
    p.set_extension("");
    page.filename = p.file_name().unwrap().to_str().unwrap().to_string();

    let content = pre_process(root, outdir, &content);
    let content = content
        + &format!(
            "\n[source](https://github.com/szabgab/rust.code-maven.com/blob/main/pages/{}.md)",
            &page.filename
        );

    let content = markdown::to_html(&content);
    //println!("{}", content);
    let content = content.replace("<h1>", "<h1 class=\"title\">");
    let content = content.replace("<h2>", "<h2 class=\"title is-4\">");
    let content = content.replace("<h3>", "<h3 class=\"title is-5\">");

    page.content = content;
    page
}

fn pre_process(root: &str, outdir: &str, text: &str) -> String {
    let re = Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").unwrap();
    let ext_to_language: HashMap<String, String> = read_languages();

    let result = re.replace_all(text, |caps: &Captures| {
        let path = Path::new(&caps[1]);
        let include_path = Path::new(root).join(path);
        if ext_to_language.contains_key(path.extension().unwrap().to_str().unwrap()) {
            let language = ext_to_language[path.extension().unwrap().to_str().unwrap()].as_str();
            include_file(include_path, path, language)
        } else {
            // TODO: we don't need to copy external images
            let output_path = Path::new(outdir).join(path);
            copy_file(&include_path, &output_path);
            caps[0].to_string() // .copy() // don't replace anything
        }
    });

    result.to_string()
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

fn include_file(include_path: PathBuf, path: &Path, language: &str) -> String {
    log::info!("include_path: {:?}", include_path);

    if include_path.exists() {
        match File::open(include_path) {
            Ok(mut file) => {
                let mut content = "".to_string();
                content += &format!(
                    "**[{}](https://github.com/szabgab/rust.code-maven.com/tree/main/{})**\n",
                    path.display(),
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
    let filename = "languages.csv";
    let mut data = HashMap::new();
    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                let parts = line.split(',');
                let parts: Vec<&str> = parts.collect();
                data.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
        Err(error) => {
            println!("Error opening file {}: {}", filename, error);
        }
    }
    data
}

#[test]
fn test_read_index() {
    let data = read_md_file("demo", "demo/pages/index.md", "temp");
    dbg!(&data);
    let expected = Page {
        title: "Index page".to_string(),
        timestamp: "2015-10-11T12:30:01".to_string(),
        description: "The text for the search engines".to_string(),
        filename: "index".to_string(),
        content: "<p>Some Text.</p>\n<p>Some more text after an empty row.</p>\n<h2 class=\"title is-4\">A title with two hash-marks</h2>\n<p>More text <a href=\"/with_todo\">with TODO</a>.</p>\n<p><a href=\"https://github.com/szabgab/rust.code-maven.com/blob/main/pages/index.md\">source</a></p>".to_string(),
        todo: vec![],
        tags: vec![],
    };
    assert_eq!(data, expected);
}

#[test]
fn test_read_todo() {
    let data = read_md_file("demo", "demo/pages/with_todo.md", "temp");
    dbg!(&data);
    let expected = Page {
        title: "Page with todos".to_string(),
        timestamp: "2023-10-11T12:30:01".to_string(),
        description: "".to_string(),
        filename: "with_todo".to_string(),
        content: "<p>Some Content.</p>\n<p><strong><a href=\"https://github.com/szabgab/rust.code-maven.com/tree/main/examples/hello_world.rs\">examples/hello_world.rs</a></strong></p>\n<pre><code class=\"language-rust\">fn main() {\n    println!(&quot;Hello World!&quot;);\n}\n\n</code></pre>\n<p><a href=\"https://github.com/szabgab/rust.code-maven.com/blob/main/pages/with_todo.md\">source</a></p>".to_string(),
        todo: vec![
            "Add another article extending on the topic".to_string(),
            "Add an article describing a prerequisite".to_string(),
        ],
        tags: vec![
            "println!".to_string(),
            "fn".to_string(),
        ],
    };
    assert_eq!(data, expected);
}

#[test]
fn test_img_with_title() {
    let data = read_md_file("demo", "demo/pages/img_with_title.md", "temp");
    dbg!(&data);
    let expected = Page {
        title: "Image with title".to_string(),
        timestamp: "2023-10-12T13:30:01".to_string(),
        description: "".to_string(),
        filename: "img_with_title".to_string(),
        content: "<p><img src=\"examples/files/code_maven_490_490.jpg\" alt=\"a title\" /></p>\n<p><a href=\"https://github.com/szabgab/rust.code-maven.com/blob/main/pages/img_with_title.md\">source</a></p>".to_string(),
        todo: vec![
        ],
        tags: vec![
            "img".to_string(),
        ],
    };
    assert_eq!(data, expected);
}
