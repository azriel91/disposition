pub use id_newtype::id;

pub use self::{
    id::{Id, IdInvalidFmt},
    map::Map,
};

mod id;
mod map;
