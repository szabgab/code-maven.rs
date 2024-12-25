use std::fs::File;
use std::io::Write as _;

pub fn new_site(root: &str) -> Result<(), String> {
    log::info!("new_site {root}");
    let path = std::path::PathBuf::from(root);
    if path.exists() {
        return Err(format!("path {root} exists"));
    }

    // TODO set the time of the pages to the generation of the site
    // TODO maybe ask questions to fill the fields and/or accept command line parameters

    let gitignore_str = include_str!("../test_cases/skeleton/.gitignore");
    let config_str = include_str!("../test_cases/skeleton/config.yaml");
    let index_str = include_str!("../test_cases/skeleton/pages/index.md");
    let about_str = include_str!("../test_cases/skeleton/pages/about.md");
    let foobar_str = include_str!("../test_cases/skeleton/authors/foobar.md");

    std::fs::create_dir_all(&path).unwrap();
    std::fs::create_dir_all(path.join("pages")).unwrap();
    std::fs::create_dir_all(path.join("authors")).unwrap();

    writeln!(
        File::create(path.join(".gitignore")).unwrap(),
        "{gitignore_str}"
    )
    .unwrap();

    writeln!(
        File::create(path.join("config.yaml")).unwrap(),
        "{config_str}"
    )
    .unwrap();

    writeln!(
        File::create(path.join("pages").join("index.md")).unwrap(),
        "{index_str}"
    )
    .unwrap();

    writeln!(
        File::create(path.join("pages").join("about.md")).unwrap(),
        "{about_str}"
    )
    .unwrap();

    writeln!(
        File::create(path.join("authors").join("foobar.md")).unwrap(),
        "{foobar_str}"
    )
    .unwrap();

    Ok(())
}
