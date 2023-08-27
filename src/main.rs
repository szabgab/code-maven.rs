use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    #[arg(long)]
    pages: String,

    #[arg(long)]
    outdir: String,
}

#[derive(Debug)]
struct Page {
    title: String,
    timestamp: String,
    content: String,
}

fn main() {
    let args = Cli::parse();
    //println!("{:?}", &args);

    if !Path::new(&args.outdir).exists() {
        fs::create_dir(&args.outdir).unwrap();
    }

    let path = Path::new(&args.pages);
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            // println!("{:?}", entry.path());
            // println!("{:?}", entry.file_name());
            let mut outfile = PathBuf::from(entry.file_name().to_owned());
            outfile.set_extension("html");
            let page = read_md_file(&entry.path().to_str().unwrap());
            dbg!(&page);
            render(page, &format!("_site/{}", outfile.display()));
        }
    }
}

fn render(page: Page, path: &str) {
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
        }
    }
}

fn read_md_file(path: &str) -> Page {
    let mut page = Page::new();
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                //dbg!(&line);
                let re = Regex::new(r"^=([a-z]+) (.*)").unwrap();
                match re.captures(&line) {
                    Some(value) => {
                        //dbg!(&value);
                        if &value[1] == "title" {
                            page.title = value[2].to_string();
                            continue;
                        }
                        if &value[1] == "timestamp" {
                            page.timestamp = value[2].to_string();
                            continue;
                        }
                    }
                    None => {}
                }
                page.content += &line;
            }
        }
        Err(error) => {
            println!("Error opening file {}: {}", path, error);
        }
    }

    page
}

#[test]
fn test_read() {
    let data = read_md_file("examples/pages/index.md");
    dbg!(&data);
    let expected = Page {
        title: "Index page".to_string(),
        timestamp: "2015-10-11T12:30:01".to_string(),
        content: "Some Text.".to_string(),
    };
    assert_eq!(data.title, expected.title);
    assert_eq!(data.timestamp, expected.timestamp);
    assert_eq!(data.content, expected.content);
}
