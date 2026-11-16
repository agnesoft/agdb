//! The source model of a Rust crate, read with [`syn`] and without compiling
//! anything: the module tree from the crate root, following `mod`
//! declarations like rustc does, over the parsed files. A [`Cfg`] selects
//! the code: all of it, or what rustc compiles with a [`CfgSet`].
//!
//! Problems with single files (unreadable, invalid, missing modules) are
//! collected as [`Error`]s and loading carries on with the rest.

mod cfg;
mod error;
mod files;
mod source_crate;

pub use cfg::Cfg;
pub use cfg::CfgSet;
pub use error::Error;
pub use files::rust_files;
pub use source_crate::FileId;
pub use source_crate::Module;
pub use source_crate::ModuleId;
pub use source_crate::SourceCrate;
pub use source_crate::SourceFile;
