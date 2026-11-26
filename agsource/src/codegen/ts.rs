//! The TypeScript target: the passes TS needs on top of the generic ones and
//! the [`Emitter`]. Enums follow serde's external tagging, `Option` is
//! `T | undefined`, every number is `number` and the data types are the
//! schema types of [`TsConfig`].

mod emitter;
mod erase_casts;
mod unescape_identifiers;

pub use emitter::Emitter;
pub use emitter::Ts;
pub use emitter::TsConfig;
pub use erase_casts::EraseCasts;
pub use unescape_identifiers::UnescapeIdentifiers;

// Re-export the shared passes that were lifted from this module to
// `codegen::pass`, so that existing `use agsource::codegen::ts::*` imports
// keep working.
pub use super::pass::AcceptElementSources;
pub use super::pass::EraseTraits;
pub use super::pass::OmitSkippedDefaults;
pub use super::pass::RemoveDeadHelpers;
pub use super::pass::RenameShadowed;
