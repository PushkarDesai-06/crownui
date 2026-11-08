/// Makes a window a session-lock surface: it covers one output while the session is locked,
/// and nothing else is shown or receives input there until the session is unlocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LockSurfaceOptions {
    /// The raw id of the output to cover, as the backend reports it (on Wayland
    /// `OutputId::raw`).
    pub output: u32,
}

impl LockSurfaceOptions {
    /// A lock surface covering the output whose raw id is `output`.
    pub const fn on_output(output: u32) -> Self {
        Self { output }
    }
}
