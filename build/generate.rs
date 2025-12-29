pub mod descriptors;
pub mod gpu;
pub mod pipelines;
pub mod samplers;

use crate::config::{Compute, Pipeline, Sampler};
use std::fmt::{Display, Formatter};

impl Display for Compute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl Display for Pipeline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl Display for Sampler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}
