use std::io;
use std::os::raw::{c_int, c_short};

/// A file descriptor entry passed to `poll(2)`.
#[repr(C)]
pub struct PollFd {
    /// The file descriptor to monitor.
    pub fd: c_int,
    /// The poll events to watch for.
    pub events: c_short,
    /// The events reported by `poll(2)`.
    pub revents: c_short,
}

/// Readable data is available.
pub const POLLIN: c_short = 0x0001;
/// An error condition was detected.
pub const POLLERR: c_short = 0x0008;
/// The peer disconnected or hung up.
pub const POLLHUP: c_short = 0x0010;
/// The file descriptor is invalid.
pub const POLLNVAL: c_short = 0x0020;

extern "C" {
    fn poll(fds: *mut PollFd, nfds: usize, timeout: c_int) -> c_int;
}

/// Wait for activity on one or more file descriptors.
pub fn wait(fds: &mut [PollFd], timeout_ms: c_int) -> io::Result<c_int> {
    let result = unsafe { poll(fds.as_mut_ptr(), fds.len(), timeout_ms) };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(result)
    }
}

impl PollFd {
    /// Create a `PollFd` entry with no reported events set.
    pub fn new(fd: c_int, events: c_short) -> Self {
        Self {
            fd,
            events,
            revents: 0,
        }
    }
}
