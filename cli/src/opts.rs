use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliOpts {
    #[clap(short, long)]
    pub config_file: Option<PathBuf>,

    #[clap(short, long)]
    pub project_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(flatten)]
    Default(Cmd),

    New(NewProject),

    #[command(subcommand)]
    Issue(Cmd),

    #[command(subcommand)]
    Project(Cmd),
}

#[derive(Subcommand, Clone)]
pub enum Cmd {
    Info {
        #[clap(short, long)]
        project: Option<String>,
    },
    List {
        #[clap(short, long)]
        project: Option<String>,
    },
    Add,
}

#[derive(Parser, Clone)]
pub struct NewProject {
    /// Create new project with manifest file
    #[arg(short, long)]
    pub manifest: bool,

    /// New project path
    pub path: PathBuf,
}
