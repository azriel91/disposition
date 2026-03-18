/// Indicates where a theme value originated.
///
/// Used to show users which values they have overridden vs. which come
/// from the base diagram.
///
/// # Variants
///
/// * `BaseDiagram` -- the value comes from `InputDiagram::base()`.
/// * `UserInput` -- the value was provided by the user's overlay diagram.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeValueSource {
    /// Value comes from the base diagram defaults.
    BaseDiagram,
    /// Value was provided by the user's input diagram.
    UserInput,
}
