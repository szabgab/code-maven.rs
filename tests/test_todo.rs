use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, ExitStatus},
};

#[test]
fn test_todo_demo() {
    let result = Command::new("cargo")
        .args(["run", "-q", "todo", "--root", "test_cases/demo"])
        .output()
        .expect("command failed to start");
    //canonicalize()
    assert_eq!(
        std::str::from_utf8(&result.stdout).unwrap(),
        "test_cases/demo/pages/with_todo.md Page with todos\n   Add another article extending on the topic\n   Add an article describing a prerequisite\n"
    );
    assert_eq!(std::str::from_utf8(&result.stderr).unwrap(), "");
    assert_eq!(result.status, ExitStatus::from_raw(0));
}

#[test]
fn test_todo_site() {
    let result = Command::new("cargo")
        .args(["run", "-q", "todo", "--root", "site"])
        .output()
        .expect("command failed to start");
    //canonicalize()
    assert_eq!(std::str::from_utf8(&result.stdout).unwrap(), "");
    assert_eq!(std::str::from_utf8(&result.stderr).unwrap(), "");
    assert_eq!(result.status, ExitStatus::from_raw(0));
}
