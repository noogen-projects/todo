use temp_testdir::TempDir;

use self::common::parse_tests_from_markdown;

mod common;

#[test]
fn new_project_test_cases() {
    let sections = parse_tests_from_markdown("tests/new_project.md").unwrap();

    for section in sections {
        let root_dir = TempDir::default();
        let mut completed_tests = Vec::new();

        println!("\n# {}", section.title);

        for test_case in section.cases {
            let test_case = test_case.with_root_dir(&root_dir.as_os_str());

            println!("Testing: {:?}", test_case.commands);
            test_case.run().unwrap();
            completed_tests.push(test_case);
        }

        // Destroy completed test cases
        drop(completed_tests);
    }
}
