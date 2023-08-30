use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use regex::Captures;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

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

#[derive(Debug, Deserialize, Serialize)]
struct Page {
    title: String,
    timestamp: String,

    #[serde(default = "get_empty_string")]
    filename: String,

    #[serde(default = "get_empty_vector")]
    todo: Vec<String>,

    #[serde(default = "get_empty_string")]
    content: String,
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

    let pages = read_pages(&args.pages, &args.root);
    render_pages(&pages, &args.outdir);
    render_archive(pages, &format!("{}/archive.html", &args.outdir))
}

fn render_archive(pages: Vec<Page>, path: &str) {
    log::info!("render archive");

    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let template_filename = String::from("templates/archive.html");
    let template = liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
        .parse_file(&template_filename)
        .unwrap();

    let globals = liquid::object!({
        "title": "Archive".to_string(),
        "pages": &pages,
    });
    let output = template.render(&globals).unwrap();

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();
}

fn render_pages(pages: &Vec<Page>, outdir: &str) {
    for page in pages {
        let mut outfile = PathBuf::from(&page.filename);
        outfile.set_extension("html");
        render(page, &format!("{}/{}", outdir, outfile.display()));
    }
}

fn read_pages(pages_path: &str, root: &str) -> Vec<Page> {
    let mut pages: Vec<Page> = vec![];
    let path = Path::new(pages_path);
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            log::info!("path: {:?}", entry.path());
            // println!("{:?}", entry.file_name());
            let page = read_md_file(root, &entry.path().to_str().unwrap());
            log::info!("{:?}", &page);
            pages.push(page);
        }
    }
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
        .parse_file(&template_filename)
        .unwrap();

    let globals = liquid::object!({
        "title": page.title,
        "content": page.content,
    });
    let output = template.render(&globals).unwrap();

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();
}

impl Page {
    pub fn new() -> Page {
        Page {
            title: "".to_string(),
            timestamp: "".to_string(),
            filename: "".to_string(),
            content: "".to_string(),
            todo: vec![],
        }
    }
}

fn read_md_file(root: &str, path: &str) -> Page {
    let mut page: Page = Page::new();

    let mut content = "".to_string();

    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut in_front_matter = false;
            let mut front_matter = "".to_string();
            for line in reader.lines() {
                let line = line.unwrap();
                log::info!("line '{}'", line);
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

    let content = pre_process(root, &content);
    let content = markdown::to_html(&content);
    //println!("{}", content);
    let content = content.replace("<h1>", "<h1 class=\"title\">");
    let content = content.replace("<h2>", "<h2 class=\"title is-4\">");
    let content = content.replace("<h3>", "<h3 class=\"title is-5\">");

    page.content = content;
    let mut p = PathBuf::from(path);
    p.set_extension("");

    page.filename = p.file_name().unwrap().to_str().unwrap().to_string();
    page
}

fn pre_process(root: &str, text: &str) -> String {
    let re = Regex::new(r"!\[\]\(([^)]+)\)").unwrap();
    let result = re.replace_all(text, |caps: &Captures| {
        let path = Path::new(&caps[1]);
        let include_path = Path::new(root).join(path);
        if path.extension().unwrap() == "rs" {
            let language = "rust";
            if include_path.exists() {
                match File::open(include_path) {
                    Ok(mut file) => {
                        let mut content = "```".to_string();
                        content += language;
                        content += "\n";
                        file.read_to_string(&mut content).unwrap();
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
        } else {
            caps[0].to_string() // .copy() // don't replace anything
        }
    });

    result.to_string()
}

#[test]
fn test_read() {
    let data = read_md_file("demo", "demo/pages/index.md");
    dbg!(&data);
    let expected = Page {
        title: "Index page".to_string(),
        timestamp: "2015-10-11T12:30:01".to_string(),
        filename: "index".to_string(),
        content: "<p>Some Text.</p>\n<p>Some more text after an empty row.</p>\n<h2 class=\"title is-4\">A title with two hash-marks</h2>\n<p>More text <a href=\"/with_todo\">with TODO</a>.</p>\n".to_string(),
        todo: vec![],
    };
    assert_eq!(data.title, expected.title);
    assert_eq!(data.timestamp, expected.timestamp);
    assert_eq!(data.content, expected.content);
    assert_eq!(data.todo, expected.todo);
    assert_eq!(data.filename, expected.filename);

    let data = read_md_file("demo", "demo/pages/with_todo.md");
    dbg!(&data);
    let expected = Page {
        title: "Page with todos".to_string(),
        timestamp: "2023-10-11T12:30:01".to_string(),
        filename: "with_todo".to_string(),
        content: "<p>Some Content.</p>\n<p><img src=\"picture.png\" alt=\"\" /></p>\n<p><img src=\"image.jpg\" alt=\"a title\" /></p>\n<pre><code class=\"language-rust\">fn main() {\n    println!(&quot;Hello World!&quot;);\n}\n</code></pre>\n".to_string(),
        todo: vec![
            "Add another article extending on the topic".to_string(),
            "Add an article describing a prerequisite".to_string(),
        ],
    };
    assert_eq!(data.title, expected.title);
    assert_eq!(data.timestamp, expected.timestamp);
    assert_eq!(data.content, expected.content);
    assert_eq!(data.todo, expected.todo);
    assert_eq!(data.filename, expected.filename);
}
