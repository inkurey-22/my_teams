use crate::commands::{parse_private_message_info, read_server_response_line};
use crate::libcli;
use std::ffi::CString;
use std::io::{self, ErrorKind, Write};
use std::net::TcpStream;
use std::os::fd::AsRawFd;
use std::os::raw::{c_int, c_short};

const STDIN_FD: c_int = 0;
const POLLIN: c_short = 0x0001;
const POLLERR: c_short = 0x0008;
const POLLHUP: c_short = 0x0010;
const POLLNVAL: c_short = 0x0020;

#[repr(C)]
struct PollFd {
    fd: c_int,
    events: c_short,
    revents: c_short,
}

unsafe extern "C" {
    fn poll(fds: *mut PollFd, nfds: usize, timeout: c_int) -> c_int;
}

pub fn wait_for_input_events(stream: &TcpStream) -> io::Result<(bool, bool)> {
    let mut fds = [
        PollFd {
            fd: STDIN_FD,
            events: POLLIN,
            revents: 0,
        },
        PollFd {
            fd: stream.as_raw_fd(),
            events: POLLIN,
            revents: 0,
        },
    ];

    let poll_result = unsafe { poll(fds.as_mut_ptr(), fds.len(), -1) };
    if poll_result < 0 {
        return Err(io::Error::last_os_error());
    }

    let stdin_ready = (fds[0].revents & POLLIN) != 0;
    let socket_ready = (fds[1].revents & (POLLIN | POLLERR | POLLHUP | POLLNVAL)) != 0;

    Ok((stdin_ready, socket_ready))
}

pub fn print_prompt() -> io::Result<()> {
    print!("myteams > ");
    io::stdout().flush()
}

pub fn drain_pending_server_infos(stream: &mut TcpStream) -> io::Result<()> {
    stream.set_nonblocking(true)?;

    loop {
        match read_server_response_line(stream) {
            Ok(line) => {
                if let Some((sender_uuid, message_body)) = parse_private_message_info(&line)? {
                    emit_private_message_event(&sender_uuid, &message_body)?;
                }
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => break,
            Err(err) => {
                stream.set_nonblocking(false)?;
                return Err(err);
            }
        }
    }

    stream.set_nonblocking(false)?;
    Ok(())
}

fn emit_private_message_event(sender_uuid: &str, message_body: &str) -> io::Result<()> {
    let sender_uuid_cstr = CString::new(sender_uuid).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "sender UUID contains null byte")
    })?;
    let message_body_cstr = CString::new(message_body).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "message body contains null byte",
        )
    })?;

    unsafe {
        let _ = libcli::client_event_private_message_received(
            sender_uuid_cstr.as_ptr(),
            message_body_cstr.as_ptr(),
        );
    }

    Ok(())
}
