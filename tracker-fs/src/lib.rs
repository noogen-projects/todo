pub use crate::tracker::FsTracker;

pub mod config;
pub mod generator;
pub mod load;
pub mod save;
pub mod tracker;

#[derive(Debug, Clone)]
pub enum Target<P> {
    WholeFile(P),
    CodeBlockInFile(P),
}

impl<P> AsRef<P> for Target<P> {
    fn as_ref(&self) -> &P {
        match self {
            Self::WholeFile(p) => p,
            Self::CodeBlockInFile(p) => p,
        }
    }
}
