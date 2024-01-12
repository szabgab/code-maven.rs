use crate::{get_pages_path, read_config, read_pages};

pub fn list_todo(root: &str, pages: &str) {
    log::info!("Read all the pages and list all the todo items");

    let config = read_config(root);
    log::info!("config");

    let pages_path = get_pages_path(root, pages);

    let (pages, _paths) = read_pages(&config, &pages_path, root);
    for page in pages {
        if !page.todo.is_empty() {
            println!(
                "{:<30} {}",
                pages_path.join(page.filename).as_os_str().to_str().unwrap(),
                page.title
            );
            for todo in page.todo {
                println!("   {}", todo);
            }
        }
    }
}
