use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, ExitStatus},
};
use tempdir::TempDir;

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

    let result = Command::new("cargo")
        .args([
            "run",
            "-q",
            "new",
            "--root",
            tmp_dir.path().join("site").to_str().unwrap(),
        ])
        .output()
        .expect("command failed to start");
    //canonicalize()
    assert_eq!(std::str::from_utf8(&result.stdout).unwrap(), "");
    assert_eq!(std::str::from_utf8(&result.stderr).unwrap(), "");
    assert_eq!(result.status, ExitStatus::from_raw(0));

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
