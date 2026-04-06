use std::io;
use std::os::raw::{c_int, c_short};

#[repr(C)]
pub struct PollFd {
    pub fd: c_int,
    pub events: c_short,
    pub revents: c_short,
}

pub const POLLIN: c_short = 0x0001;
pub const POLLERR: c_short = 0x0008;
pub const POLLHUP: c_short = 0x0010;
pub const POLLNVAL: c_short = 0x0020;

extern "C" {
    fn poll(fds: *mut PollFd, nfds: usize, timeout: c_int) -> c_int;
}

pub fn wait(fds: &mut [PollFd], timeout_ms: c_int) -> io::Result<c_int> {
    let result = unsafe { poll(fds.as_mut_ptr(), fds.len(), timeout_ms) };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(result)
    }
}

impl PollFd {
    pub fn new(fd: c_int, events: c_short) -> Self {
        Self {
            fd,
            events,
            revents: 0,
        }
    }
}
