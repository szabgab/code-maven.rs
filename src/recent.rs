use chrono::{DateTime, Duration, Utc};

use crate::{get_pages_path, read_config, read_pages, Config, Page, ToPath};

pub fn get_recent(root: &str, path_to_pages: &str, days: &str) {
    log::info!("get_recent");

    let config = read_config(root);
    let url = &config.url;

    let pages_path = get_pages_path(root, path_to_pages);

    let (pages, _paths) = read_pages(&config, &pages_path, root);

    list_recent(&config, pages, days, url);
}

#[allow(clippy::print_stdout)]
fn list_recent(config: &Config, pages: Vec<Page>, recent: &str, url: &str) {
    if recent.is_empty() {
        return;
    }
    log::info!("render email");

    let days: i64 = recent.parse().unwrap();
    let now: DateTime<Utc> = Utc::now();
    let date = now - Duration::days(days);
    let date = date.format("%Y-%m-%dT%H:%M:%S").to_string();
    //println!("{:?}", pages);

    let filtered_pages: Vec<&Page> = pages
        .iter()
        .filter(|page| page.filename != "index" && page.filename != "archive")
        .filter(|page| page.timestamp > date)
        .collect();

    let template = include_str!("../templates/email.html");
    let template = liquid::ParserBuilder::with_stdlib()
        .filter(ToPath)
        .build()
        .unwrap()
        .parse(template)
        .unwrap();

    let globals = liquid::object!({
        "pages": &filtered_pages,
        "config": config,
        "url": url,
    });
    let output = template.render(&globals).unwrap();
    println!("{output}");
}