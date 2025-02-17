use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{anyhow, Context};
use regex::Regex;

pub fn split_command_parts(command_line: &str) -> Vec<&str> {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""([^"]+)"|\S+"#).expect("regex must be correct"));

    REGEX
        .find_iter(command_line)
        .map(|found| found.as_str().trim_matches('"'))
        .collect()
}

#[derive(Debug)]
pub enum Cmd {
    Cd(PathBuf),
    Ls(PathBuf),
    Mkdir(Vec<PathBuf>),
    Rm(Vec<PathBuf>),
    Cat(PathBuf),
    Echo(String, Option<PathBuf>),
}

pub enum CmdResponse {
    Success,
    ChangeDirTo(PathBuf),
    Output(String),
}

impl Cmd {
    pub fn parse<'a>(root_dir: impl AsRef<Path>, source: &'a str) -> Result<Self, Vec<&'a str>> {
        let root_dir = root_dir.as_ref();
        let parts = split_command_parts(source);

        let cmd = match &parts[..] {
            ["cd", path] => Self::Cd(root_dir.join(path)),
            ["ls", path] => Self::Ls(root_dir.join(path)),
            ["mkdir", pathes @ ..] => Self::Mkdir(pathes.iter().map(|path| root_dir.join(path)).collect()),
            ["rm", pathes @ ..] => Self::Rm(pathes.iter().map(|path| root_dir.join(path)).collect()),
            ["cat", path] => Self::Cat(root_dir.join(path)),
            ["echo", text @ .., ">", path] => Self::Echo(
                text.iter().map(|item| *item).collect::<Vec<_>>().join(" "),
                Some(root_dir.join(path)),
            ),
            ["echo", text @ ..] => Self::Echo(text.iter().map(|item| *item).collect::<Vec<_>>().join(" "), None),
            _ => return Err(parts),
        };
        Ok(cmd)
    }

    pub fn run(self) -> anyhow::Result<CmdResponse> {
        match self {
            Self::Cd(path) => cd(path),
            Self::Ls(path) => ls(path),
            Self::Mkdir(pathes) => mkdir(pathes),
            Self::Rm(pathes) => rm(pathes),
            Self::Cat(path) => cat(path),
            Self::Echo(text, path) => echo(text, path),
        }
    }
}

fn cd(path: PathBuf) -> anyhow::Result<CmdResponse> {
    if path.is_dir() {
        Ok(CmdResponse::ChangeDirTo(path))
    } else {
        Err(anyhow!("Path `{}` is not dir", path.display()))
    }
}

fn ls(path: PathBuf) -> anyhow::Result<CmdResponse> {
    let mut output = String::new();
    for entry in fs::read_dir(&path)? {
        output.push_str(&format!("{} ", entry?.path().strip_prefix(&path)?.display()));
    }

    if let Some(last) = output.pop() {
        if last == ' ' {
            output.push('\n');
        } else {
            output.push(last);
        }
    }

    Ok(CmdResponse::Output(output))
}

fn mkdir(pathes: Vec<PathBuf>) -> anyhow::Result<CmdResponse> {
    for path in pathes {
        fs::create_dir_all(&path).with_context(|| format!("Failed to create directory `{}`", path.display()))?;
    }
    Ok(CmdResponse::Success)
}

fn rm(pathes: Vec<PathBuf>) -> anyhow::Result<CmdResponse> {
    for path in pathes {
        fs::remove_dir_all(&path).with_context(|| format!("Failed to remove directory `{}`", path.display()))?;
    }
    Ok(CmdResponse::Success)
}

fn cat(path: PathBuf) -> anyhow::Result<CmdResponse> {
    let content = fs::read_to_string(&path).with_context(|| format!("Failed to read file `{}`", path.display()))?;
    Ok(CmdResponse::Output(content))
}

fn echo(text: String, path: Option<PathBuf>) -> anyhow::Result<CmdResponse> {
    if let Some(path) = path {
        fs::write(&path, text).with_context(|| format!("Failed to write file `{}`", path.display()))?;
        Ok(CmdResponse::Success)
    } else {
        Ok(CmdResponse::Output(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
