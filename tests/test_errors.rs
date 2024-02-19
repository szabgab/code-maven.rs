use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, ExitStatus},
};
//use tempdir::TempDir;

#[test]
fn test_read_pages() {
    //let tmp_dir = TempDir::new("example").unwrap();

    let result = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--",
            "web",
            "--root",
            "test_cases/demo",
            "--pages",
            "/some/foo/bar",
        ])
        .output()
        .expect("command failed to start");

    assert!(std::str::from_utf8(&result.stdout).unwrap().contains(
        "read_dir of '\"/some/foo/bar\"' call failed: No such file or directory (os error 2)"
    ));
    assert_eq!(std::str::from_utf8(&result.stderr).unwrap(), "");
    assert_eq!(result.status, ExitStatus::from_raw(256 * 1));
}
