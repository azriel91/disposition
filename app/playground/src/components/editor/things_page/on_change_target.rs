//! Target map selector for generic key-value row mutations.
//!
//! Re-exports [`OnChangeTarget`] from `disposition_input_rt` so that existing
//! callers within the playground continue to compile without path changes.

pub use disposition_input_rt::on_change_target::OnChangeTarget;
