use std::fmt::Debug;

pub trait MessageData: Clone + Debug + Send
{}