//! The source model of a Rust crate, read with [`syn`] and without compiling
//! anything: the module tree from the crate root, following `mod`
//! declarations like rustc does, over the parsed files. A [`Cfg`] selects
//! the code: all of it, or what rustc compiles with a [`CfgSet`].
//! [`SourceCrate::resolve`] tells what a path names, following the imports
//! of the modules like rustc does. [`SourceCrate::reachable`] collects
//! what the [`SourceCrate::exports`] given as roots lead to.
//!
//! [`packages`] lists the crates of a cargo workspace, from `cargo metadata`.
//!
//! Problems with single files (unreadable, invalid, missing modules,
//! unresolved imports) are
//! collected as [`Error`]s and loading carries on with the rest.

mod cfg;
mod error;
mod files;
mod package;
mod reachable;
mod resolve;
mod source_crate;

pub use cfg::Cfg;
pub use cfg::CfgSet;
pub use error::Error;
pub use files::rust_files;
pub use package::Package;
pub use package::Target;
pub use package::TargetKind;
pub use package::packages;
pub use reachable::Export;
pub use reachable::Reachable;
pub use resolve::Namespace;
pub use resolve::Resolved;
pub use source_crate::FileId;
pub use source_crate::ItemId;
pub use source_crate::Module;
pub use source_crate::ModuleId;
pub use source_crate::SourceCrate;
pub use source_crate::SourceFile;
