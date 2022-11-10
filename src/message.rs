use std::{fmt::Debug, hash::Hash};

pub trait Message : Copy + Debug + Eq + Hash
{
}