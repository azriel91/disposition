//! Commonizes card-level state initialization and keyboard handling for
//! collapsible card components.
//!
//! Card components across the editor share the same initialization pattern
//! for `collapsed`, `rename_target`, `can_move_up`, and `can_move_down`
//! signals/fields, as well as the same `onkeydown` match structure for
//! `CardKeyAction`. This module extracts that boilerplate into reusable
//! helpers.

use dioxus::{
    core::Event,
    hooks::use_signal,
    html::KeyboardData,
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::components::editor::{
    common::RenameRefocusTarget,
    keyboard_nav::{self, CardKeyAction},
    reorderable::is_rename_target,
};

use super::RenameRefocus;

/// Groups common card-level logic: state initialization and keyboard
/// handling.
///
/// Mirrors the pattern used by [`FieldNav`](super::FieldNav) for field-level
/// helpers.
pub struct CardComponent;

/// Holds the common signals and flags used by collapsible card components.
///
/// Created via [`CardComponent::state_init`] or
/// [`CardComponent::state_init_with_rename`].
pub struct CardState {
    /// Whether the card is collapsed.
    pub collapsed: Signal<bool>,

    /// Which refocus target the next ID rename should use.
    ///
    /// Only meaningful for cards that support ID renaming. For cards
    /// that don't, this signal is still present but unused.
    pub rename_target: Signal<RenameRefocusTarget>,

    /// Whether the card can be moved up in the list (i.e. `index > 0`).
    pub can_move_up: bool,

    /// Whether the card can be moved down in the list (i.e. `index + 1 <
    /// entry_count`).
    pub can_move_down: bool,
}

impl CardComponent {
    /// Initializes a [`CardState`] for a card that does **not** support ID
    /// renaming.
    ///
    /// The card starts collapsed. Use this for cards like
    /// `CssClassPartialsCard` and `StyleAliasesSection`.
    ///
    /// # Parameters
    ///
    /// * `index`: zero-based position of this card in the list.
    /// * `entry_count`: total number of entries in the list.
    pub fn state_init(index: usize, entry_count: usize) -> CardState {
        let collapsed = use_signal(|| true);
        let rename_target = use_signal(|| RenameRefocusTarget::IdInput);

        CardState {
            collapsed,
            rename_target,
            can_move_up: index > 0,
            can_move_down: index + 1 < entry_count,
        }
    }

    /// Initializes a [`CardState`] for a card that supports ID renaming.
    ///
    /// When `rename_refocus` matches `card_id`, the card starts expanded so
    /// the user can see the renamed entry. Use this for cards like
    /// `ProcessCard`, `TagThingsCard`, and `EdgeGroupCard`.
    ///
    /// # Parameters
    ///
    /// * `index`: zero-based position of this card in the list.
    /// * `entry_count`: total number of entries in the list.
    /// * `rename_refocus`: signal carrying the post-rename refocus context.
    /// * `card_id`: the current ID of this card, used to check against
    ///   `rename_refocus`.
    pub fn state_init_with_rename(
        index: usize,
        entry_count: usize,
        rename_refocus: Signal<Option<RenameRefocus>>,
        card_id: &str,
    ) -> CardState {
        let collapsed = use_signal({
            let is_target = is_rename_target(rename_refocus, card_id);
            move || !is_target
        });
        let rename_target = use_signal(|| RenameRefocusTarget::IdInput);

        CardState {
            collapsed,
            rename_target,
            can_move_up: index > 0,
            can_move_down: index + 1 < entry_count,
        }
    }

    /// Returns an `onkeydown` handler for a collapsible card.
    ///
    /// The returned closure delegates to [`keyboard_nav::card_keydown`] and
    /// handles the common `Collapse`, `Expand`, `Toggle`, `EnterEdit`, and
    /// `None` actions internally. The caller-specific `MoveUp` and
    /// `MoveDown` actions are forwarded to the provided closures.
    ///
    /// # Parameters
    ///
    /// * `data_attr`: the `data-*` attribute placed on the card wrapper, e.g.
    ///   `"data-process-card"`, `"data-edge-group-card"`.
    /// * `card_state`: the [`CardState`] for the card.
    /// * `on_move_up`: closure to call when the user presses **Alt+Up** and
    ///   `can_move_up` is `true`.
    /// * `on_move_down`: closure to call when the user presses **Alt+Down** and
    ///   `can_move_down` is `true`.
    pub fn card_onkeydown(
        data_attr: &'static str,
        card_state: CardState,
        mut on_move_up: impl FnMut() + 'static,
        mut on_move_down: impl FnMut() + 'static,
    ) -> impl FnMut(Event<KeyboardData>) {
        let CardState {
            mut collapsed,
            rename_target: _,
            can_move_up,
            can_move_down,
        } = card_state;

        move |evt: Event<KeyboardData>| {
            let action = keyboard_nav::card_keydown(evt, data_attr);
            match action {
                CardKeyAction::MoveUp => {
                    if can_move_up {
                        on_move_up();
                    }
                }
                CardKeyAction::MoveDown => {
                    if can_move_down {
                        on_move_down();
                    }
                }
                CardKeyAction::Collapse => collapsed.set(true),
                CardKeyAction::Expand => collapsed.set(false),
                CardKeyAction::Toggle => {
                    let is_collapsed = *collapsed.read();
                    collapsed.set(!is_collapsed);
                }
                CardKeyAction::EnterEdit => collapsed.set(false),
                CardKeyAction::None => {}
            }
        }
    }
}
