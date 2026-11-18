//! The declarations of a crate in an intermediate representation
//! ([`ir`]), from [`SourceCrate::ir`](crate::SourceCrate::ir),
//! Rust source from it ([`rust`]) and the check of the two against the
//! source ([`compare`]). For generators of other languages the IR of a
//! builder API is lowered ([`lower`]) into [`lowered`], rewritten by the
//! [`pass`]es the generator lists and dumped with [`print`](mod@print).

pub mod compare;
pub mod ir;
mod load;
pub mod lower;
pub mod lowered;
pub mod pass;
pub mod print;
pub mod rust;
#[cfg(test)]
mod test_utilities;
