use serde::{Serialize};

pub trait MessageTrait: Sized + Serialize {}
