use crate::{get_pages_path, read_config, read_pages};

pub fn list_todo(root: &str, path_to_pages: &str) -> Result<(), String> {
    log::info!("Read all the pages and list all the todo items");

    let config = read_config(root)?;
    log::info!("config");

    let pages_path = get_pages_path(root, path_to_pages);

    let pages = read_pages(&config, &pages_path, root);
    for page in pages {
        if page.redirect.is_some() {
            continue;
        }
        #[expect(clippy::print_stdout)]
        if !page.todo.is_empty() {
            println!(
                "{:<30} {}",
                pages_path.join(page.filename).as_os_str().to_str().unwrap(),
                page.title
            );
            #[expect(clippy::print_stdout)]
            for todo in page.todo {
                println!("   {todo}");
            }
        }
    }

    Ok(())
}
