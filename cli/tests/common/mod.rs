use std::io;
use std::path::Path;

use temp_testdir::TempDir;

use self::case::TestSection;

pub mod case;
pub mod cmd;

pub const CARGO_BIN_ALIAS: &str = "todo";

pub fn parse_tests_from_markdown(md_file_path: impl AsRef<Path>) -> io::Result<Vec<TestSection>> {
    case::parse_tests_from_markdown(md_file_path, Some(CARGO_BIN_ALIAS))
}

pub fn run_test_cases(md_file_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let sections = parse_tests_from_markdown(md_file_path)?;

    for section in sections {
        let test_dir = TempDir::default();
        let mut completed_tests = Vec::new();

        println!("\n# {}", section.title);

        for test_case in section.cases {
            let test_case = test_case.with_test_dir(&test_dir.as_os_str());

            println!("Testing: {:?}", test_case.commands);
            test_case.run()?;
            completed_tests.push(test_case);
        }

        // Destroy completed test cases
        drop(completed_tests);
    }
    Ok(())
}
