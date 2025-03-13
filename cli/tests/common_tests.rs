use common::case::TestCase;
use common::cmd::split_command_parts;

#[allow(dead_code)]
mod common;

#[test]
fn split_command() {
    assert_eq!(split_command_parts("mkdir a b c"), vec!["mkdir", "a", "b", "c"]);
    assert_eq!(split_command_parts("cd a/b cd \"ef g\""), vec![
        "cd", "a/b", "cd", "ef g"
    ]);
    assert_eq!(split_command_parts("echo \"test A\""), vec!["echo", "test A"]);
    assert_eq!(split_command_parts("echo a \"b c d\" ef"), vec![
        "echo", "a", "b c d", "ef"
    ]);
}

#[test]
fn parse_test_case() {
    let test = TestCase::parse(
        r#"
$ todo new "test A"
    Creating `test A` project
"#,
        None,
        None,
    );

    assert_eq!(test.commands.len(), 1);
    assert_eq!(test.commands[0], "todo new \"test A\"");
    assert_eq!(test.output.text, "    Creating `test A` project\n");

    let test = TestCase::parse(
        r#"
# Some comment
$ todo new "test A"
    Creating `test A` project"#,
        None,
        None,
    );

    assert_eq!(test.commands.len(), 1);
    assert_eq!(test.commands[0], "todo new \"test A\"");
    assert_eq!(test.output.text, "    Creating `test A` project");

    let test = TestCase::parse(
        r#"
# Some comment

$ mkdir "test A"
$ todo new "test A"
    Creating `test A` project
Error: destination `~/test A` already exists
"#,
        None,
        None,
    );

    assert_eq!(test.commands.len(), 2);
    assert_eq!(test.commands[0], "mkdir \"test A\"");
    assert_eq!(test.commands[1], "todo new \"test A\"");
    assert_eq!(
        test.output.text,
        "    Creating `test A` project\nError: destination `~/test A` already exists\n"
    );
}
