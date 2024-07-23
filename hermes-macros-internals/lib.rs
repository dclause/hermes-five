extern crate proc_macro2;
extern crate quote;
extern crate syn;

pub use internals::*;

#[cfg_attr(build_from_git, path = "../hermes-macros/src/internals/mod.rs")]
#[cfg_attr(not(build_from_git), path = "src/mod.rs")]
mod internals;
