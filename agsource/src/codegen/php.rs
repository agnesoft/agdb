//! The PHP target: the [`Emitter`] for PHP 8.1+. Enums follow serde's
//! external tagging: unit-only enums are `BackedEnum`, payload enums are
//! classes with static factory methods. `Option` is `T|null`, `int` and
//! `float` are distinct, and the data types implement `\JsonSerializable`.

mod emitter;

pub use emitter::Emitter;
pub use emitter::Php;
pub use emitter::PhpConfig;
