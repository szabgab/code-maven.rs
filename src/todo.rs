use crate::{get_pages_path, read_config, read_pages};

pub fn list_todo(root: &str, pages: &str) {
    log::info!("Read all the pages and list all the todo items");

    let config = read_config(root);
    log::info!("config");

    let pages_path = get_pages_path(root, pages);

    // TODO remove the need to pass the outdir to the read_pages
    let outdir = "_site";
    let (pages, _paths) = read_pages(&config, &pages_path, root, outdir);
    for page in pages {
        if !page.todo.is_empty() {
            println!("{}", page.title);
            for todo in page.todo {
                println!("   {}", todo);
            }
        }
    }
}
