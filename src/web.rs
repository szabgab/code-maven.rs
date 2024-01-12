use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};

use crate::{filter_words, get_pages_path, read_config, read_pages, topath, Config, Page, ToPath};

pub type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

type Tags = HashMap<String, i32>;
const IMG: &str = "img";

pub fn web(root: &str, pages: &str, outdir: &str, email: &str) {
    log::info!("Generate pages for web site");

    if !Path::new(outdir).exists() {
        fs::create_dir(outdir).unwrap();
    }
    let tags_dir = Path::new(outdir).join("tags");
    if !Path::new(&tags_dir).exists() {
        fs::create_dir(tags_dir).unwrap();
    }

    let images_dir = Path::new(outdir).join(IMG);
    if !Path::new(&images_dir).exists() {
        fs::create_dir(images_dir).unwrap();
    }

    let config = read_config(root);
    log::info!("config");
    let url = &config.url;
    log::info!("pages_path");

    let pages_path = get_pages_path(root, pages);

    let pages = read_pages(&config, &pages_path, root, outdir);
    let tags: Tags = collect_tags(&pages);
    render_pages(&config, &pages, outdir, url);
    render_tag_pages(&config, &pages, &tags, outdir, url);
    render_sitemap(&pages, &format!("{}/sitemap.xml", outdir), url);
    render_atom(&config, &pages, &format!("{}/atom", outdir), url);
    render_archive(&config, &pages, outdir, url);
    render_robots_txt(&format!("{}/robots.txt", outdir), url);
    render_email(
        &config,
        pages,
        &format!("{}/email.html", outdir),
        email,
        url,
    );
}

fn render_email(config: &Config, pages: Vec<Page>, path: &str, email: &str, url: &str) {
    if email.is_empty() {
        return;
    }
    log::info!("render email");

    let days: i64 = email.parse().unwrap();
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

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();
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

fn render_sitemap(pages: &[Page], path: &str, url: &str) {
    log::info!("render sitemap");
    let template = include_str!("../templates/sitemap.xml");
    let template = liquid::ParserBuilder::with_stdlib()
        .filter(ToPath)
        .build()
        .unwrap()
        .parse(template)
        .unwrap();

    let pages: Vec<&Page> = pages.iter().filter(|page| page.published).collect();

    let globals = liquid::object!({
        "pages": &pages,
        "url": url,
    });
    let output = template.render(&globals).unwrap();

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();
}

fn render_atom(config: &Config, pages: &[Page], path: &str, url: &str) {
    log::info!("render atom feed");
    let pages: Vec<&Page> = pages.iter().filter(|page| page.published).collect();

    let template = include_str!("../templates/atom.xml");
    let template = liquid::ParserBuilder::with_stdlib()
        .filter(ToPath)
        .build()
        .unwrap()
        .parse(template)
        .unwrap();

    let globals = liquid::object!({
        "pages": &pages,
        "url": url,
        "site_name": config.site_name,
        "author_name": "Gábor Szabó",
        "updated": pages[0].timestamp,
    });
    let output = template.render(&globals).unwrap();

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();
}

fn render_archive(config: &Config, pages: &[Page], outdir: &str, url: &str) {
    log::info!("render archive");

    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let filtered_pages: Vec<&Page> = pages
        .iter()
        .filter(|page| page.published)
        .filter(|page| page.filename != "index" && page.filename != "archive")
        .collect();
    let template = include_str!("../templates/archive.html");
    let template = liquid::ParserBuilder::with_stdlib()
        .filter(ToPath)
        .partials(partials)
        .build()
        .unwrap()
        .parse(template)
        .unwrap();

    let globals = liquid::object!({
        "title": config.archive.title,
        "description": config.archive.description,
        "keywords": vec!["archive"], // TODO use something from config
        "pages": &filtered_pages,
        "config": config,
        "url": url,
        "pagepath": "archive",
        "site_name": config.site_name,
    });
    let output = template.render(&globals).unwrap();

    let path = Path::new(outdir).join("archive.html");
    log::info!("archive file {:?}", path);
    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();

    let image_file = PathBuf::from(outdir).join(IMG).join("archive.png");

    let banner = banner_builder::Banner {
        width: 1000,
        height: 500,
        text: config.archive.title.clone(),
        background_color: "FFFFFF".to_owned(),
    };
    banner_builder::draw_image(&banner, &image_file);
}

fn render_tag_pages(config: &Config, pages: &Vec<Page>, tags: &Tags, outdir: &str, url: &str) {
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

        let globals = liquid::object!({
            "title": format!("Articles tagged with '{}'", tag),
            "description": format!("Articles about Rust tagged with '{}'", tag),
            "keywords": vec![""], // TODO: include tag, but make sure we only put there letters and numbers
            "pages": pages_with_tag,
            "config": config,
            "url": url,
            "pagepath": format!("tags/{}", topath(tag)),
            "site_name": config.site_name,
        });

        let path = Path::new(outdir).join("tags").join(topath(tag));
        log::info!("render_tag {}", tag);

        render_any(include_str!("../templates/tag.html"), path, globals);
    }

    let mut tags: Vec<_> = tags.keys().collect();
    tags.sort();

    let globals = liquid::object!({
        "title": config.tags.title,
        "description": config.tags.description,
        "keywords": vec!["tags"], // TODO use something from config
        "tags": tags,
        "config": config,
        "url": url,
        "pagepath": "tags/",
        "site_name": config.site_name,
    });

    render_any(
        include_str!("../templates/tags.html"),
        Path::new(outdir).join("tags").join("index"),
        globals,
    );
}

fn render_any(template: &str, mut path: PathBuf, globals: liquid::Object) {
    path.set_extension("html");

    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let template = liquid::ParserBuilder::with_stdlib()
        .filter(ToPath)
        .partials(partials)
        .build()
        .unwrap()
        .parse(template)
        .unwrap();

    let output = template.render(&globals).unwrap();

    log::info!("saving file at {:?}", path);
    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();
}

fn render_pages(config: &Config, pages: &Vec<Page>, outdir: &str, url: &str) {
    for page in pages {
        if page.filename == "archive" {
            continue;
        }

        let mut outfile = PathBuf::from(&page.filename);
        outfile.set_extension("html");
        render_single_page(config, page, outfile, outdir, url);
    }
}

pub fn load_templates() -> Result<Partials, Box<dyn Error>> {
    // log::info!("load_templates");

    let mut partials = Partials::empty();
    partials.add(
        "templates/incl/header.html",
        include_str!("../templates/incl/header.html"),
    );
    partials.add(
        "templates/incl/footer.html",
        include_str!("../templates/incl/footer.html"),
    );
    partials.add(
        "templates/incl/navigation.html",
        include_str!("../templates/incl/navigation.html"),
    );
    partials.add(
        "templates/incl/google.html",
        include_str!("../templates/incl/google.html"),
    );
    Ok(partials)
}

pub fn render_single_page(config: &Config, page: &Page, outfile: PathBuf, outdir: &str, url: &str) {
    let path = Path::new(outdir).join(outfile);

    log::info!("render path {:?}", path);

    // let image_file = image_file.join(IMG);
    // let mut image_file = image_file.join(&page.filename);
    // image_file.set_extension("png");
    let mut image_path = PathBuf::from(IMG).join(&page.filename);
    image_path.set_extension("png");
    let image_file = PathBuf::from(outdir).join(&image_path);
    // log::warn!("{} {:?}", page.filename, image_file);
    // log::warn!("{}", page.title);
    let banner = banner_builder::Banner {
        width: 1000,
        height: 500,
        text: page.title.clone(),
        background_color: "FFFFFF".to_owned(),
    };
    let image = banner_builder::draw_image(&banner, &image_file);

    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let template = include_str!("../templates/page.html");
    let template = liquid::ParserBuilder::with_stdlib()
        .filter(ToPath)
        .partials(partials)
        .build()
        .unwrap()
        .parse(template)
        .unwrap();

    let mut footer = config.footer.clone();

    if config.link_to_source {
        footer = format!(
            "{} [source]({}/blob/{}/pages/{}.md)",
            footer, config.repo, config.branch, &page.filename
        );
    }
    let footer = markdown::to_html(&footer);

    let globals = liquid::object!({
        "title": page.title,
        "description": page.description,
        "keywords": filter_words(&page.tags),
        "content": page.content,
        "page": page,
        "pagepath": page.filename,
        "config": config,
        "footer": footer,
        "url": url,
        "image": image,
        "image_path": image_path,
        "site_name": config.site_name,
    });
    let output = template.render(&globals).unwrap();

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{}", output).unwrap();
}