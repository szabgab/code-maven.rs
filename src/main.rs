use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    #[arg(long)]
    pages: String,

    #[arg(long)]
    outdir: String,
}

#[derive(Debug, Deserialize)]
struct Page {
    title: String,
    timestamp: String,

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

    let path = Path::new(&args.pages);
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            log::info!("path: {:?}", entry.path());
            // println!("{:?}", entry.file_name());
            let mut outfile = PathBuf::from(entry.file_name().to_owned());
            outfile.set_extension("html");
            let page = read_md_file(&entry.path().to_str().unwrap());
            log::info!("{:?}", &page);
            render(page, &format!("{}/{}", &args.outdir, outfile.display()));
        }
    }
}

fn render(page: Page, path: &str) {
    log::info!("render path {}", path);
    let template_filename = String::from("templates/page.html");
    let template = liquid::ParserBuilder::with_stdlib()
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
            content: "".to_string(),
            todo: vec![],
        }
    }
}

fn read_md_file(path: &str) -> Page {
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

    let content = markdown::to_html(&content);
    //println!("{}", content);
    let content = content.replace("<h1>", "<h1 class=\"title\">");
    let content = content.replace("<h2>", "<h2 class=\"title is-4\">");
    let content = content.replace("<h3>", "<h3 class=\"title is-5\">");

    page.content = content;
    page
}

#[test]
fn test_read() {
    let data = read_md_file("examples/pages/index.md");
    dbg!(&data);
    let expected = Page {
        title: "Index page".to_string(),
        timestamp: "2015-10-11T12:30:01".to_string(),
        content: "<p>Some Text.</p>\n<p>Some more text after an empty row.</p>\n<h2 class=\"title is-4\">A title with two hash-marks</h2>\n<p>More text</p>\n".to_string(),
        todo: vec![],
    };
    assert_eq!(data.title, expected.title);
    assert_eq!(data.timestamp, expected.timestamp);
    assert_eq!(data.content, expected.content);
    assert_eq!(data.todo, expected.todo);

    let data = read_md_file("examples/pages/with_todo.md");
    dbg!(&data);
    let expected = Page {
        title: "Page with todos".to_string(),
        timestamp: "2023-10-11T12:30:01".to_string(),
        content: "<p>Some Content.</p>\n".to_string(),
        todo: vec![
            "Add another article extending on the topic".to_string(),
            "Add an article describing a prerequisite".to_string(),
        ],
    };
    assert_eq!(data.title, expected.title);
    assert_eq!(data.timestamp, expected.timestamp);
    assert_eq!(data.content, expected.content);
    assert_eq!(data.todo, expected.todo);
}
