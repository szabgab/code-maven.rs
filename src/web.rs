#![allow(clippy::std_instead_of_core)]

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;

use feed_rs::parser;

use crate::{
    collect_backlinks, copy_files, filter_words, get_files_to_copy, get_pages_path, markdown_pages,
    read_config, read_config_file, read_pages, topath, Author, Config, Page, ToPath,
};

use crate::curly::{check_for_invalid_curly_code, process_curly_tags};

pub type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

type Tags = HashMap<String, i32>;
const IMG: &str = "img";

pub fn web(root: &str, config_path: &str, path_to_pages: &str, outdir: &str) -> Result<(), String> {
    log::info!("Generate pages for web site");

    if !Path::new(outdir).exists() {
        fs::create_dir_all(outdir).unwrap();
    }
    let tags_dir = Path::new(outdir).join("tags");
    if !Path::new(&tags_dir).exists() {
        fs::create_dir_all(tags_dir).unwrap();
    }

    let images_dir = Path::new(outdir).join(IMG);
    if !Path::new(&images_dir).exists() {
        fs::create_dir_all(images_dir).unwrap();
    }

    let config = if config_path.is_empty() {
        read_config(root)?
    } else {
        read_config_file(PathBuf::from(config_path), root)?
    };
    log::info!("config");
    let url = &config.url;
    log::info!("pages_path");

    let pages_path = get_pages_path(root, path_to_pages);

    let pages = read_pages(&config, &pages_path, root);
    let pages = collect_backlinks(pages);
    let paths = get_files_to_copy(&pages).unwrap();
    let pages = process_curly_tags(&config, root, pages);
    check_for_invalid_curly_code(&pages);
    let pages = markdown_pages(pages);

    let tags: Tags = collect_tags(&pages);
    copy_files(root, outdir, &paths);
    copy_files(
        root,
        outdir,
        &config
            .authors
            .iter()
            .map(|author| PathBuf::from("images").join(author.picture.clone()))
            .filter(|path| path.exists())
            .collect::<Vec<PathBuf>>(),
    );

    render_pages(&config, &pages, outdir, url);
    render_tag_pages(&config, &pages, &tags, outdir, url);
    render_sitemap(&pages, &format!("{outdir}/sitemap.xml"), url);
    render_atom(&config, &pages, &format!("{outdir}/atom.xml"), url)?;
    render_archive(&config, &pages, outdir, url);
    render_robots_txt(&format!("{outdir}/robots.txt"), url);

    Ok(())
}

fn collect_tags(pages: &Vec<Page>) -> Tags {
    log::info!("collect_tags");

    let mut tags: Tags = HashMap::new();
    for page in pages {
        if page.redirect.is_some() {
            continue;
        }
        if !page.published {
            continue;
        }
        for tag in &page.tags {
            tags.insert(tag.to_lowercase(), 1);
        }
    }
    tags
}

fn render_robots_txt(path: &str, url: &str) {
    log::info!("render_robots_txt");

    let text = format!("Sitemap: {url}/sitemap.xml\n\nUser-agent: *\n");

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{text}").unwrap();
}

fn render_sitemap(pages: &[Page], path: &str, url: &str) {
    log::info!("render_sitemap");

    let template = include_str!("../templates/sitemap.xml");
    let template = liquid::ParserBuilder::with_stdlib()
        .filter(ToPath)
        .build()
        .unwrap()
        .parse(template)
        .unwrap();

    let pages: Vec<&Page> = pages
        .iter()
        .filter(|page| page.published)
        .filter(|page| page.redirect.is_none())
        .collect();

    let globals = liquid::object!({
        "pages": &pages,
        "url": url,
    });
    let output = template.render(&globals).unwrap();

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{output}").unwrap();
}

fn render_atom(config: &Config, pages: &[Page], path: &str, url: &str) -> Result<(), String> {
    log::info!("render atom feed");
    let pages: Vec<&Page> = pages
        .iter()
        .filter(|page| page.published)
        .filter(|page| page.redirect.is_none())
        .filter(|page| page.url_path != "archive") // exclude archive
        //.filter(|page| !page.url_path.is_empty()) // exclude main page
        //.filter(|page| !page.title.is_empty())
        .collect();

    let pages = match &config.atom {
        Some(atom) => {
            if pages.len() < atom.max {
                &*pages
            } else {
                &pages[0..atom.max]
            }
        }
        None => &*pages,
    };

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
        "author_name": config.author_name,
        "updated": pages[0].timestamp,
    });
    let output = template.render(&globals).unwrap();

    match parser::parse(output.as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            return Err(format!("Parsing feed failed with error {err}"));
        }
    };

    let mut file = File::create(path).unwrap();
    writeln!(&mut file, "{output}").unwrap();
    Ok(())
}

fn render_archive(config: &Config, pages: &[Page], outdir: &str, url: &str) {
    log::info!("render_archive");

    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {error}"),
    };

    let filtered_pages: Vec<&Page> = pages
        .iter()
        .filter(|page| page.published)
        .filter(|page| page.redirect.is_none())
        .filter(|page| !page.url_path.is_empty() && page.url_path != "archive")
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
    writeln!(&mut file, "{output}").unwrap();

    let image_file = PathBuf::from(outdir).join(IMG).join("archive.png");

    let banner = banner_builder::Banner {
        width: 1000,
        height: 500,
        text: config.archive.title.clone(),
        background_color: "FFFFFF".to_owned(),
        embed: vec![],
        lines: vec![],
        size: 24,
    };

    // Currently we don't embed any images in the banner
    // so we can use any path.
    let root = Path::new(".");
    banner_builder::draw_image(&banner, root, &image_file);
}

fn render_tag_pages(config: &Config, pages: &Vec<Page>, tags: &Tags, outdir: &str, url: &str) {
    log::info!("render_tag_pages");

    #[expect(clippy::iter_over_hash_type)]
    for tag in tags.keys() {
        if tag == ".." {
            log::error!("We cannot save a file for a tag of 2 dots: '..'");
            continue;
        }
        if tag.contains('/') && tag != "/" {
            log::error!("For now we don't save tags with / in them:: '{tag}'");
            continue;
        }
        // if !tag.chars().all(char::is_alphanumeric) {
        //     log::error!("For now we don't save tags with non alphanumeric: '{tag}'");
        //     continue;
        // }
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
        Err(error) => panic!("Error loading templates {error}"),
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
    let mut file = File::create(&path).unwrap_or_else(|_| panic!("Could not create file {path:?}"));
    writeln!(&mut file, "{output}").unwrap();
}

fn render_pages(config: &Config, pages: &Vec<Page>, outdir: &str, url: &str) {
    log::info!("render_pages");

    for page in pages {
        if page.url_path == "archive" {
            continue;
        }

        let mut outfile = PathBuf::from(&page.filename);
        outfile.set_extension("html");
        if page.redirect.is_none() {
            render_and_save_single_page(config, page, outfile, outdir, url);
        } else {
            render_and_save_redirect_page(page, outfile, outdir);
        }
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

pub fn render_and_save_single_page(
    config: &Config,
    page: &Page,
    outfile: PathBuf,
    outdir: &str,
    url: &str,
) {
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
        embed: vec![],
        lines: vec![],
        size: 24,
    };

    // Currently we don't embed any images in the banner
    // so we can use any path.
    let root = Path::new(".");

    let image = banner_builder::draw_image(&banner, root, &image_file);

    match render_single_page(config, page, url, image, image_path) {
        Ok(output) => {
            let mut file = File::create(path).unwrap();
            writeln!(&mut file, "{output}").unwrap();
        }
        Err(err) => {
            log::error!("{err}");
            std::process::exit(1);
        }
    };
}

fn render_and_save_redirect_page(page: &Page, outfile: PathBuf, outdir: &str) {
    let path = Path::new(outdir).join(outfile);
    log::info!("render redirect page path {:?}", path);
    let output = format!(
        r#"<meta http-equiv="refresh" content="0; url={}" />"#,
        page.redirect.as_ref().unwrap()
    );

    writeln!(File::create(path).unwrap(), "{output}").unwrap();
}

fn render_single_page(
    config: &Config,
    page: &Page,
    url: &str,
    image: bool,
    image_path: PathBuf,
) -> Result<String, String> {
    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => return Err(format!("Error loading templates {error}")),
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
            "{} [source]({}/blob/{}/pages/{})",
            footer, config.repo, config.branch, &page.filename
        );
    }
    let footer = markdown::to_html(&footer);

    let author = if page.author.is_empty() {
        Author {
            name: String::new(),
            nickname: String::new(),
            picture: String::new(),
            text: String::new(),
        }
    } else {
        let authors = config
            .authors
            .iter()
            .filter(|author| author.nickname == page.author)
            .collect::<Vec<&Author>>();

        if authors.is_empty() {
            return Err(format!(
                "The nickname '{}' used in the file '{}' is not in the config.yaml file.",
                page.author, page.filename
            ));
        }

        authors[0].clone()
    };

    let globals = liquid::object!({
        "title": page.title,
        "description": page.description,
        "keywords": filter_words(&page.tags),
        "content": page.content,
        "page": page,
        "pagepath": page.url_path,
        "config": config,
        "footer": footer,
        "url": url,
        "image": image,
        "image_path": image_path,
        "site_name": config.site_name,
        "author": author,
    });
    Ok(template.render(&globals).unwrap())
}

#[test]
fn test_show_author_text() {
    use crate::read_md_file;

    let config = read_config("test_cases/config_with_authors/").unwrap();
    let page = read_md_file(
        &config,
        "test_cases/config_with_authors/",
        "test_cases/config_with_authors/pages/index.md",
    )
    .unwrap();
    let url = &config.url;
    let image = false;
    let image_path = PathBuf::from("");
    let output = render_single_page(&config, &page, url, image, image_path).unwrap();
    assert!(output.contains("Gabor Szabo"));
    assert!(output.contains("the author of the Rust Maven web site"));
    //assert_eq!(output, "");
}

#[test]
fn test_bad_author() {
    use crate::read_md_file;

    let config = read_config("test_cases/config_with_authors/").unwrap();
    let page = read_md_file(
        &config,
        "test_cases/config_with_authors/",
        "test_cases/config_with_authors/pages/bad_author.md",
    )
    .unwrap();
    let url = &config.url;
    let image = false;
    let image_path = PathBuf::from("");
    let error = render_single_page(&config, &page, url, image, image_path)
        .err()
        .unwrap();
    assert_eq!(
        error,
        "The nickname 'george' used in the file 'bad_author.md' is not in the config.yaml file."
    );
}
