//! Common types shared between `disposition_model` and `disposition_ir`.

#[macro_use]
extern crate id_newtype;

pub use id_newtype::id;

pub use self::{
    id::{Id, IdInvalidFmt},
    map::Map,
};

pub mod edge;
pub mod entity;
pub mod theme;

mod id;
mod map;
