//! Common types shared between `disposition_input_model` and
//! `disposition_ir_model`.

#[macro_use]
extern crate id_newtype;

pub use id_newtype::id;

pub use self::{
    id::{Id, IdInvalidFmt},
    map::Map,
    set::Set,
};

pub mod edge;
pub mod entity;
pub mod theme;

mod id;
mod map;
mod set;
