//! The declarations of a crate in an intermediate representation
//! ([`ir`]), from [`SourceCrate::ir`](crate::SourceCrate::ir),
//! Rust source from it ([`rust`]) and the check of the two against the
//! source ([`compare`]).

pub mod compare;
pub mod ir;
mod load;
pub mod rust;
