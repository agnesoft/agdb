#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CollidedValue<T> {
    pub value: T,
}

impl<T> CollidedValue<T> {
    pub fn new(value: T) -> Self {
        CollidedValue { value }
    }
}
