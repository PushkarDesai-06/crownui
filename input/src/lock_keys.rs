/// Lock keys toggled on at the time of an event, as the DOM reports through
/// `getModifierState("CapsLock")` and `getModifierState("NumLock")`. Kept apart from
/// [`Modifiers`](crate::Modifiers) so a lit Caps Lock never changes which keybind matches.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LockKeys {
    /// Caps Lock is on.
    pub caps_lock: bool,
    /// Num Lock is on.
    pub num_lock: bool,
}
