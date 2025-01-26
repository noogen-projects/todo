use std::fs;
use std::hash::Hash;
use std::path::Path;

use serde::Serialize;

use crate::config::{ProjectConfig, SaveConfigError};
use crate::Target;

pub fn project<ID: Serialize + Hash + Eq>(
    config: &ProjectConfig<ID>,
    destination: Target<impl AsRef<Path>>,
) -> Result<(), SaveConfigError> {
    match destination {
        Target::WholeFile(path) => config.save(path),
        Target::CodeBlockInFile(path) => {
            let prefix = "```toml project";
            let suffix = "```";
            let content = format!("{prefix}\n{}\n{suffix}", config.to_toml()?.trim_end());

            if path.as_ref().exists() {
                let existing_content = fs::read_to_string(path.as_ref())?;

                let mut new_content = String::new();
                let mut inserted = false;
                let mut in_block = false;
                for line in existing_content.lines() {
                    if line.trim().starts_with('`') && line.trim().to_lowercase() == prefix {
                        in_block = true;
                        new_content.push_str(&content);
                        inserted = true;
                    }
                    if !in_block {
                        new_content.push_str(line);
                        new_content.push('\n');
                    }
                    if line.trim().starts_with('`') && line.trim().to_lowercase() == suffix {
                        in_block = false;
                    }
                }
                if !inserted {
                    new_content.push_str(&content);
                }

                fs::write(path, new_content).map_err(Into::into)
            } else {
                let project_name = config.name.as_deref().unwrap_or("");
                fs::write(path, format!("# {project_name}\n\n{content}")).map_err(Into::into)
            }
        },
    }
}
