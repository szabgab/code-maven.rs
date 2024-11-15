use std::{
    fs,
    os::unix::process::ExitStatusExt,
    process::{Command, ExitStatus},
};
use tempdir::TempDir;

fn new_site(root: &str) {
    let result = Command::new("cargo")
        .args(["run", "-q", "new", "--root", root])
        .output()
        .expect("command failed to start");
    //canonicalize()
    assert_eq!(std::str::from_utf8(&result.stdout).unwrap(), "");
    assert_eq!(std::str::from_utf8(&result.stderr).unwrap(), "");
    assert_eq!(result.status, ExitStatus::from_raw(0));
}

fn generate_site(root: &str, outdir: &str) -> (ExitStatus, String, String) {
    let result = Command::new("cargo")
        .args(["run", "-q", "web", "--root", root, "--outdir", outdir])
        .output()
        .expect("command failed to start");
    //canonicalize()
    (
        result.status,
        std::str::from_utf8(&result.stdout).unwrap().to_owned(),
        std::str::from_utf8(&result.stderr).unwrap().to_owned(),
    )
}

#[test]
fn test_new_without_parameters() {
    // TODO create external test that will create a new blog using the "new" command
    let result = Command::new("cargo")
        .args(["run", "-q", "--", "new"])
        .output()
        .expect("command failed to start");

    assert_eq!(std::str::from_utf8(&result.stdout).unwrap(), "");
    assert_eq!(
        std::str::from_utf8(&result.stderr).unwrap(),
        "error: the following required arguments were not provided:\n  --root <ROOT>\n\nUsage: code-maven new --root <ROOT>\n\nFor more information, try '--help'.\n"
    );
    assert_eq!(result.status, ExitStatus::from_raw(256 * 2));
}

#[test]
fn test_new_with_parameters() {
    let tmp_dir = TempDir::new("example").unwrap();
    println!("tempdir: {:?}", tmp_dir);

    new_site(tmp_dir.path().join("site").to_str().unwrap());

    let content = tmp_dir
        .path()
        .read_dir()
        .unwrap()
        .map(|de| de.unwrap().file_name().to_str().unwrap().to_owned())
        .collect::<Vec<String>>();
    assert_eq!(content, ["site"]);

    let mut content = tmp_dir
        .path()
        .join("site")
        .read_dir()
        .unwrap()
        .map(|de| de.unwrap().file_name().to_str().unwrap().to_owned())
        .collect::<Vec<String>>();
    content.sort();
    assert_eq!(content, [".gitignore", "authors", "config.yaml", "pages"]);

    let mut content = tmp_dir
        .path()
        .join("site")
        .join("authors")
        .read_dir()
        .unwrap()
        .map(|de| de.unwrap().file_name().to_str().unwrap().to_owned())
        .collect::<Vec<String>>();
    content.sort();
    assert_eq!(content, ["foobar.md"]);

    let mut content = tmp_dir
        .path()
        .join("site")
        .join("pages")
        .read_dir()
        .unwrap()
        .map(|de| de.unwrap().file_name().to_str().unwrap().to_owned())
        .collect::<Vec<String>>();
    content.sort();
    assert_eq!(content, ["about.md", "index.md"])
}

#[test]
fn test_new_generate() {
    let tmp_dir = TempDir::new("example").unwrap();
    println!("tempdir: {:?}", tmp_dir);

    new_site(tmp_dir.path().join("site").to_str().unwrap());

    let outdir = tmp_dir.path().join("out");
    assert!(!outdir.exists());
    let (exit, out, err) = generate_site(
        tmp_dir.path().join("site").to_str().unwrap(),
        outdir.to_str().unwrap(),
    );
    assert_eq!(out, "");
    assert_eq!(err, "");
    assert_eq!(exit, ExitStatus::from_raw(0));

    assert!(outdir.exists());
    assert!(outdir.join("index.html").exists());
    assert!(outdir.join("robots.txt").exists());
    assert!(outdir.join("sitemap.xml").exists());
    assert!(outdir.join("atom.xml").exists());
    assert!(outdir.join("archive.html").exists());
    assert!(outdir.join("about.html").exists());

    assert!(outdir.join("tags").join("about.html").exists());
    assert!(outdir.join("tags").join("blog.html").exists());
    assert!(outdir.join("tags").join("index.html").exists());

    assert!(outdir.join("img").join("about.png").exists());
    assert!(outdir.join("img").join("archive.png").exists());
    assert!(outdir.join("img").join("index.png").exists());
}

#[test]
fn test_page_author_not_in_config() {
    let tmp_dir = TempDir::new("example").unwrap();
    println!("tempdir: {:?}", tmp_dir);

    let root = tmp_dir.path().join("site");

    new_site(root.to_str().unwrap());
    let source_path = "test_cases/author-not-in-config.md";
    let destination_path = root.join("pages").join("author-not-in-config.md");
    fs::copy(source_path, destination_path).unwrap();

    let outdir = tmp_dir.path().join("out");

    assert!(!outdir.exists());
    let (exit, out, err) = generate_site(root.to_str().unwrap(), outdir.to_str().unwrap());
    assert!(out.contains("The nickname 'george' used in the file 'author-not-in-config.md' is not in the config.yaml file."));
    assert_eq!(err, "");
    assert_eq!(exit, ExitStatus::from_raw(256));
}

#[test]
fn test_page_invalid_curly_code() {
    let tmp_dir = TempDir::new("example").unwrap();
    println!("tempdir: {:?}", tmp_dir);

    let root = tmp_dir.path().join("site");

    new_site(root.to_str().unwrap());
    let source_path = "test_cases/invalid_curly.md";
    let destination_path = root.join("pages").join("invalid_curly.md");
    fs::copy(source_path, destination_path).unwrap();

    let outdir = tmp_dir.path().join("out");

    assert!(!outdir.exists());
    let (exit, out, err) = generate_site(root.to_str().unwrap(), outdir.to_str().unwrap());
    assert_eq!(out, "");
    //assert!(out.contains("Invalid curly code '{% opening liquid tag' in 'invalid_curly.md'"));
    assert!(err.contains("{% opening liquid tag"));
    assert_eq!(exit, ExitStatus::from_raw(25856));
    //assert_eq!(err, "");
    //assert_eq!(exit, ExitStatus::from_raw(256));
}
