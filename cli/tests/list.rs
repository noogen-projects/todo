use self::common::run_test_cases;

mod common;

#[test]
fn list_test_cases() {
    run_test_cases("tests/list.md").unwrap();
}
