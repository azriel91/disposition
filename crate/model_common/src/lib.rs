//! Common types shared between `disposition_model` and `disposition_ir`.

#[macro_use]
extern crate id_newtype;

pub use id_newtype::id;

pub use self::{
    id::{Id, IdInvalidFmt},
    map::Map,
};

mod id;
mod map;
