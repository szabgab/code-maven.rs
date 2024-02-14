use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, ExitStatus},
};

#[test]
fn test_drafts_demo() {
    let result = Command::new("cargo")
        .args(["run", "-q", "drafts", "--root", "test_cases/demo"])
        .output()
        .expect("command failed to start");

    assert_eq!(
        std::str::from_utf8(&result.stdout).unwrap(),
        "\n---- Drafts ----\ndraft.md                       Draft page\n"
    );
    assert_eq!(std::str::from_utf8(&result.stderr).unwrap(), "");
    assert_eq!(result.status, ExitStatus::from_raw(0));
}

#[test]
fn test_drafts_site() {
    //simple_logger::init_with_level(log::Level::Debug).unwrap();
    let result = Command::new("cargo")
        .args(["run", "-q", "drafts", "--root", "site"])
        .output()
        .expect("command failed to start");

    assert_eq!(
        std::str::from_utf8(&result.stdout).unwrap(),
        "\n---- Drafts ----\n"
    );
    assert_eq!(std::str::from_utf8(&result.stderr).unwrap(), "");
    assert_eq!(result.status, ExitStatus::from_raw(0));
}
