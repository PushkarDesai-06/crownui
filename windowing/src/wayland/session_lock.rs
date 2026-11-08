use sctk::{
    reexports::client::{Connection, QueueHandle, globals::GlobalList},
    session_lock::{
        SessionLock, SessionLockHandler, SessionLockState, SessionLockSurface,
        SessionLockSurfaceConfigure,
    },
};

use super::{
    error::Error,
    event_loop::{QueuedEvent, WaylandPlatform},
    window::surface_window_id,
};

const MANAGER_INTERFACE: &str = "ext_session_lock_manager_v1";

/// How the session lock this platform asked for is going.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionLockEvent {
    /// Every output is covered by lock surfaces; the session is locked.
    Locked,
    /// The compositor refused the lock (another client holds one) or ended it; the lock is gone
    /// and must not be unlocked.
    Finished,
}

/// Where the platform's session lock stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionLockStatus {
    /// No lock requested.
    Unlocked,
    /// Lock requested, waiting for [`SessionLockEvent::Locked`] or [`SessionLockEvent::Finished`].
    Pending,
    /// The compositor confirmed the lock.
    Locked,
}

pub(crate) fn bind_manager(
    globals: &GlobalList,
    qh: &QueueHandle<WaylandPlatform>,
) -> Option<SessionLockState> {
    let advertised = globals.contents().with_list(|list| {
        list.iter()
            .any(|global| global.interface == MANAGER_INTERFACE)
    });
    advertised.then(|| SessionLockState::new(globals, qh))
}

impl WaylandPlatform {
    /// Asks the compositor to lock the session. Create one window per output with
    /// [`LockSurfaceOptions`](crate::LockSurfaceOptions) right away; the outcome arrives as
    /// [`WaylandEvent::SessionLock`](crate::wayland::WaylandEvent::SessionLock).
    pub fn lock_session(&mut self) -> Result<(), Error> {
        if self.session_lock.is_some() {
            return Ok(());
        }
        let manager = self
            .standard
            .session_lock
            .as_ref()
            .ok_or(Error::MissingGlobal(MANAGER_INTERFACE))?;
        let lock = manager
            .lock(&self.queue_handle)
            .map_err(|_| Error::MissingGlobal(MANAGER_INTERFACE))?;
        self.session_lock = Some(lock);
        Ok(())
    }

    /// Unlocks a confirmed lock and flushes the request, so it reaches the compositor even if
    /// the process exits right after. Returns `false` (and changes nothing) unless the lock is
    /// confirmed; a pending lock cannot be unlocked by the protocol.
    pub fn unlock_session(&mut self) -> bool {
        let Some(lock) = self.session_lock.as_ref().filter(|lock| lock.is_locked()) else {
            return false;
        };
        lock.unlock();
        self.session_lock = None;
        let _ = self.connection.flush();
        true
    }

    pub fn session_lock_status(&self) -> SessionLockStatus {
        match &self.session_lock {
            None => SessionLockStatus::Unlocked,
            Some(lock) if lock.is_locked() => SessionLockStatus::Locked,
            Some(_) => SessionLockStatus::Pending,
        }
    }

    pub(crate) fn active_session_lock(&self) -> Option<&SessionLock> {
        self.session_lock.as_ref()
    }
}

impl SessionLockHandler for WaylandPlatform {
    fn locked(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {
        self.pending_events
            .push(QueuedEvent::SessionLock(SessionLockEvent::Locked));
    }

    fn finished(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {
        self.session_lock = None;
        self.pending_events
            .push(QueuedEvent::SessionLock(SessionLockEvent::Finished));
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        surface: SessionLockSurface,
        configure: SessionLockSurfaceConfigure,
        _: u32,
    ) {
        let id = surface_window_id(surface.wl_surface());
        if let Some(window) = self.windows.get_mut(&id) {
            window.configure_lock(configure.new_size, &mut self.pending_events);
        }
    }
}
