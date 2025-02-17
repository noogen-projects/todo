use std::io;
use std::path::Path;

use self::case::TestSection;

pub mod case;
pub mod cmd;

pub const CARGO_BIN_ALIAS: &str = "todo";

pub fn parse_tests_from_markdown(md_file_path: impl AsRef<Path>) -> io::Result<Vec<TestSection>> {
    case::parse_tests_from_markdown(md_file_path, Some(CARGO_BIN_ALIAS))
}
