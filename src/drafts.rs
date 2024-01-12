use crate::{get_pages_path, read_config, read_pages};

pub fn list_drafts(root: &str, pages: &str) {
    log::info!("Read all the pages and list the ones that are not published");

    let config = read_config(root);
    log::info!("config");

    let pages_path = get_pages_path(root, pages);

    // TODO remove the need to pass the outdir to the read_pages
    let outdir = "_site";
    let pages = read_pages(&config, &pages_path, root, outdir);
    println!("\n---- Drafts ----");
    for page in pages {
        if !page.published {
            println!("{:<30} {}", page.filename, page.title);
        }
    }
}
