use crate::{include_tag, latest_tag, youtube_tag, Config, Page};

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
                        process_curly_tags_for_text(config, root, row, &all_pages).unwrap()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
            page
        })
        .collect::<Vec<Page>>();
    pages
}

fn process_curly_tags_for_text(
    config: &Config,
    root: &str,
    text: &str,
    all_pages: &[Page],
) -> Result<String, liquid_core::Error> {
    let template = liquid::ParserBuilder::with_stdlib()
        .tag(latest_tag::LatestTag)
        .tag(youtube_tag::YoutubeTag)
        .tag(include_tag::IncludeTag)
        .build()?
        .parse(text)?;

    let globals = liquid::object!({"items": all_pages, "branch": config.branch, "repo": config.repo , "root": root});

    template.render(&globals)
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
