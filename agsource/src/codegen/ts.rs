//! The TypeScript target: the passes TS needs on top of the generic ones and
//! the [`Emitter`]. Enums follow serde's external tagging, `Option` is
//! `T | undefined`, every number is `number` and the data types are the
//! schema types of [`TsConfig`].

mod accept_element_sources;
mod emitter;
mod erase_casts;
mod erase_traits;
mod omit_skipped_defaults;
mod remove_dead_helpers;
mod rename_shadowed;
mod unescape_identifiers;

pub use accept_element_sources::AcceptElementSources;
pub use emitter::Emitter;
pub use emitter::Ts;
pub use emitter::TsConfig;
pub use erase_casts::EraseCasts;
pub use erase_traits::EraseTraits;
pub use omit_skipped_defaults::OmitSkippedDefaults;
pub use remove_dead_helpers::RemoveDeadHelpers;
pub use rename_shadowed::RenameShadowed;
pub use unescape_identifiers::UnescapeIdentifiers;
