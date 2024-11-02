use std::collections::HashMap;
use std::path::Path;

use regex::{Captures, Regex};

use crate::{include_file, read_languages, Config, Page};

pub fn process_curly_tags(config: &Config, root: &str, pages: Vec<Page>) -> Vec<Page> {
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
                        let row = process_curly_tags_for_text(row, &all_pages);
                        let row = process_curly_tags_youtube(&row);
                        process_curly_include(config, root, &row)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
            page
        })
        .collect::<Vec<Page>>();
    pages
}

fn process_curly_tags_youtube(text: &str) -> String {
    let re = Regex::new(r#"\{%\s+youtube\s+id="([^"]+)"\s+%\}"#).unwrap();
    re.replace_all(text, r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/$1" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe>"#).to_string()
}

fn process_curly_tags_for_text(text: &str, all_pages: &[Page]) -> String {
    let re = Regex::new(r#"\{%\s+latest\s+limit=(\d+)\s+(?:tag="([^"]+)"\s+)?%\}"#).unwrap();
    re.replace_all(text, |caps: &Captures| {
        let mut count = 0;
        let limit = caps[1].parse::<usize>().unwrap();
        let tag = caps.get(2);

        let mut html = String::new();
        #[expect(clippy::needless_range_loop)]
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

fn process_curly_include(config: &Config, root: &str, text: &str) -> String {
    log::info!("process_curly_include for string {text:?}");
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

pub fn check_for_invalid_curly_code(pages: &Vec<Page>) {
    for page in pages {
        let mut in_code = false;
        for row in page.content.split('\n') {
            if row.starts_with("```") {
                in_code = !in_code;
                continue;
            }
            if !in_code && row.contains("{%") {
                log::error!("Invalid curly code '{}' in '{}'", row, page.filename);
                std::process::exit(1);
            }
        }
    }
}
