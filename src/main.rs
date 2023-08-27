use std::fs::File;
use std::io::{BufRead, BufReader};

use regex::Regex;

fn main() {
    let data = read_md_file("examples/pages/index.md");
    dbg!(&data);
}

#[derive(Debug)]
struct Page {
    title: String,
}

impl Page {
    pub fn new() -> Page {
        Page {
            title: "".to_string(),
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
                        }
                    }
                    None => {}
                }
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
    };
    assert_eq!(data.title, expected.title);
}
