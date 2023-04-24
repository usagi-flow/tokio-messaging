use std::fmt::Debug;
use std::hash::Hash;

pub trait Message: Copy + Debug + Eq + Hash {}
