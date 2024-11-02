use crate::{get_pages_path, read_config, read_pages};

#[expect(clippy::print_stdout)]
pub fn list_drafts(root: &str, path_to_pages: &str) -> Result<(), String> {
    log::info!("Read all the pages and list the ones that are not published");

    let config = read_config(root)?;
    log::info!("config");

    let pages_path = get_pages_path(root, path_to_pages);

    let pages = read_pages(&config, &pages_path, root);
    println!("\n---- Drafts ----");
    for page in pages {
        #[expect(clippy::print_stdout)]
        if !page.published {
            println!("{:<30} {}", page.filename, page.title);
        }
    }
    Ok(())
}
