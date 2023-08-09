//! This module implements a Semigroup trait for several types used by stest.
//!
//! Semigroups are a common abstraction in functional program. They effectively mean that a type is
//! combinable. A combinable type can easily be folded or reduced in the context of an iterator.
//!
//! See: https://en.wikipedia.org/wiki/Semigroup
//! See: https://typelevel.org/cats/typeclasses/semigroup.html
//! See: https://hackage.haskell.org/package/base-4.15.0.0/docs/Data-Semigroup.html

/// Proves a type is combinable.
///
/// A combinable type can be easily folded or reduced in the context of an iterator.
pub trait Semigroup {
    fn combine(self, other: Self) -> Self;
}

/// Proves Result<A, B> is a semigroup if A is a semigroup.
///
/// Results are combined by sequencing them and either combining their successful, "left" side if
/// they're both successful or else returning the first error in the sequence.
impl<A: Semigroup, B> Semigroup for Result<A, B> {
    fn combine(self, other: Result<A, B>) -> Result<A, B> {
        self.and_then(|a| other.map(|other_a| a.combine(other_a)))
    }
}

/// Proves Vec<A> is a semigroup.
///
/// Vectors are combined by appending the other vector onto the one whose combine method is called.
impl<A> Semigroup for Vec<A> {
    fn combine(mut self: Vec<A>, mut other: Vec<A>) -> Vec<A> {
        self.append(&mut other);
        self
    }
}
