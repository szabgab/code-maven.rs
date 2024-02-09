use std::{
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

fn generate_site(root: &str, outdir: &str) {
    let result = Command::new("cargo")
        .args(["run", "-q", "web", "--root", root, "--outdir", outdir])
        .output()
        .expect("command failed to start");
    //canonicalize()
    assert_eq!(std::str::from_utf8(&result.stdout).unwrap(), "");
    assert_eq!(std::str::from_utf8(&result.stderr).unwrap(), "");
    assert_eq!(result.status, ExitStatus::from_raw(0));
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
    assert_eq!(content, ["authors", "config.yaml", "pages"]);

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
    generate_site(
        tmp_dir.path().join("site").to_str().unwrap(),
        outdir.to_str().unwrap(),
    );
    assert!(outdir.exists());
    assert!(outdir.join("index.html").exists());
    assert!(outdir.join("robots.txt").exists());
    assert!(outdir.join("sitemap.xml").exists());
    assert!(outdir.join("atom").exists());
    assert!(outdir.join("archive.html").exists());
    assert!(outdir.join("about.html").exists());

    assert!(outdir.join("tags").join("about.html").exists());
    assert!(outdir.join("tags").join("blog.html").exists());
    assert!(outdir.join("tags").join("index.html").exists());

    assert!(outdir.join("img").join("about.png").exists());
    assert!(outdir.join("img").join("archive.png").exists());
    assert!(outdir.join("img").join("index.png").exists());
}
