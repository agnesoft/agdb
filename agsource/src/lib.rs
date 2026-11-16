//! The source model of a Rust crate, read with [`syn`] and without compiling
//! anything: the module tree from the crate root, following `mod`
//! declarations like rustc does, over the parsed files.
//!
//! Problems with single files (unreadable, invalid, missing modules) are
//! collected as [`Error`]s and loading carries on with the rest.

mod error;
mod files;
mod source_crate;

pub use error::Error;
pub use files::rust_files;
pub use source_crate::FileId;
pub use source_crate::Module;
pub use source_crate::ModuleId;
pub use source_crate::SourceCrate;
pub use source_crate::SourceFile;
