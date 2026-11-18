//! The passes over the [`lowered`](super::lowered) IR. Each generator lists
//! the passes its target needs in [`Passes`], the built-in ones and its
//! own.

mod erase_newtypes;
mod erase_ownership;
mod explicit_returns;
mod infer_empty_lists;
mod inline;
mod lower_conversions;
mod prune;
mod resolve;
mod trait_dicts;

pub use erase_newtypes::EraseNewtypes;
pub use erase_ownership::EraseOwnership;
pub use explicit_returns::ExplicitReturns;
pub use infer_empty_lists::InferEmptyLists;
pub use inline::Inline;
pub use lower_conversions::LowerConversions;
pub use lower_conversions::normalizer;
pub use prune::Prune;
pub use resolve::Resolve;
pub use resolve::returned_ty;
pub use resolve::subst;
pub use trait_dicts::TraitDicts;
pub use trait_dicts::dict_field;
pub use trait_dicts::dict_global;

use super::lowered::Module;
use crate::Error;

/// A rewrite of the module.
pub trait Pass {
    /// The name of the pass in the dumps.
    fn name(&self) -> &str;

    fn run(&self, m: &mut Module, errors: &mut Vec<Error>);
}

/// The passes of a generator in the order they run.
#[derive(Default)]
pub struct Passes {
    passes: Vec<Box<dyn Pass>>,
}

impl Passes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, pass: impl Pass + 'static) -> Self {
        self.passes.push(Box::new(pass));
        self
    }

    /// Runs the passes, with `dump` called with the name of each pass and
    /// the module after it.
    pub fn run(&self, m: &mut Module, dump: &mut dyn FnMut(&str, &Module)) -> Vec<Error> {
        let mut errors = Vec::new();

        for pass in &self.passes {
            pass.run(m, &mut errors);
            dump(pass.name(), m);
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Named(&'static str);

    impl Pass for Named {
        fn name(&self) -> &str {
            self.0
        }

        fn run(&self, m: &mut Module, errors: &mut Vec<Error>) {
            m.builders.push(self.0.to_owned());
            errors.push(Error::Lowering {
                path: Default::default(),
                line: 0,
                item: self.0.to_owned(),
                message: "ran".into(),
            });
        }
    }

    mod passes {
        use super::*;

        mod run {
            use super::*;

            #[test]
            fn run_should_run_the_passes_in_order_and_dump_after_each() {
                let passes = Passes::new().with(Named("a")).with(Named("b"));
                let mut m = Module::default();
                let mut dumps = Vec::new();

                let errors = passes.run(&mut m, &mut |name, m| {
                    dumps.push((name.to_owned(), m.builders.clone()));
                });

                assert_eq!(
                    dumps,
                    [
                        ("a".to_owned(), vec!["a".to_owned()]),
                        ("b".to_owned(), vec!["a".to_owned(), "b".to_owned()])
                    ]
                );
                assert_eq!(
                    errors.iter().map(ToString::to_string).collect::<Vec<_>>(),
                    ["`a`: ran", "`b`: ran"]
                );
            }
        }
    }
}
