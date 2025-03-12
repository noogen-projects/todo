use std::path::PathBuf;

use clap::{Parser, Subcommand};
use todo_app::issue;
use todo_app::target::Location;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliOpts {
    #[arg(short, long)]
    pub config_file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(flatten)]
    Default(Cmd),

    New(NewProject),

    Add(AddIssue),

    #[command(subcommand)]
    Issue(Cmd),

    #[command(subcommand)]
    Project(Cmd),
}

#[derive(Subcommand, Clone)]
pub enum Cmd {
    Info {
        #[arg(short, long)]
        project: Option<String>,
    },
    List {
        #[arg(short, long)]
        project: Option<String>,
    },
}

#[derive(Parser, Clone)]
pub struct NewProject {
    /// Create new project with manifest file
    #[arg(short = 'm', long, conflicts_with = "with_project_config")]
    pub with_manifest: bool,

    /// Create new project with project config file
    #[arg(long, conflicts_with = "with_manifest")]
    pub with_project_config: bool,

    /// New project location (new project path by example)
    pub location: String,
}

#[derive(Parser)]
pub struct AddIssue {
    /// The location of the project to add issue (current directory project by default)
    #[command(flatten)]
    pub location: ProjectLocation,

    /// The issue adding order
    #[command(flatten)]
    pub order: Order,

    /// The text of the issue
    pub issue: String,
}

#[derive(Parser, Clone, Copy)]
pub struct Order {
    /// Issue will be added to the top of the list
    #[arg(short, long)]
    pub first: bool,

    /// Issue will be added to the bottom of the list
    #[arg(short, long)]
    pub last: bool,
}

impl Order {
    pub fn into_order(self) -> Option<issue::Order> {
        if self.first {
            Some(issue::Order::First)
        } else if self.last {
            Some(issue::Order::Last)
        } else {
            None
        }
    }
}

#[derive(Parser)]
pub struct ProjectLocation {
    /// Project location (path, id or name)
    #[arg(short, long, conflicts_with_all = &["project_path", "project_id", "project_name"])]
    pub project: Option<String>,

    /// Project path, using if project location is not specified
    #[arg(long, conflicts_with_all = &["project", "project_id", "project_name"])]
    pub project_path: Option<PathBuf>,

    /// Project id, using if `--project` and `--project-path` is not specified
    #[arg(long, conflicts_with_all = &["project", "project_path", "project_name"])]
    pub project_id: Option<String>,

    /// Project name, using if `--project`, `--project-path` and `--project-id` is not specified
    #[arg(long, conflicts_with_all = &["project", "project_path", "project_id"])]
    pub project_name: Option<String>,
}

impl ProjectLocation {
    pub fn into_location(self) -> Option<Location> {
        let Self {
            project,
            project_path,
            project_id,
            project_name,
        } = self;

        project
            .map(Location::from_unknown)
            .or(project_path.map(Location::Path))
            .or(project_id.map(Location::Id))
            .or(project_name.map(Location::Name))
    }
}
