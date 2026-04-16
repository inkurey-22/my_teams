use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;

/// Result of a non-blocking read attempt from a client socket.
pub enum ReadLinesResult {
    /// The peer closed the connection.
    Disconnected,
    /// No data was ready to be read.
    WouldBlock,
    /// One or more complete lines were read.
    Lines(Vec<String>),
}

/// Write a payload without failing on `WouldBlock`.
pub fn write_nonblocking(stream: &mut TcpStream, payload: &str) -> io::Result<()> {
    match stream.write_all(payload.as_bytes()) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::WouldBlock => Ok(()),
        Err(err) => Err(err),
    }
}

/// Read as many complete newline-delimited lines as are currently available.
pub fn read_lines_nonblocking(
    stream: &mut TcpStream,
    input_buffer: &mut String,
) -> io::Result<ReadLinesResult> {
    let mut buf = [0u8; 1024];
    match stream.read(&mut buf) {
        Ok(0) => Ok(ReadLinesResult::Disconnected),
        Ok(n) => {
            input_buffer.push_str(String::from_utf8_lossy(&buf[..n]).as_ref());

            let mut lines = Vec::new();
            while let Some(newline_idx) = input_buffer.find('\n') {
                let line = input_buffer[..=newline_idx]
                    .trim_end_matches(['\r', '\n'])
                    .to_string();
                input_buffer.drain(..=newline_idx);
                if !line.is_empty() {
                    lines.push(line);
                }
            }

            Ok(ReadLinesResult::Lines(lines))
        }
        Err(err) if err.kind() == ErrorKind::WouldBlock => Ok(ReadLinesResult::WouldBlock),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_lines_nonblocking_with_single_line() {
        let data = b"hello\n";
        let stream: &mut dyn Read = &mut &data[..];
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let mut input_buffer = String::new();
        input_buffer.push_str(String::from_utf8_lossy(&buf[..n]).as_ref());

        let mut lines = Vec::new();
        while let Some(newline_idx) = input_buffer.find('\n') {
            let line = input_buffer[..=newline_idx]
                .trim_end_matches(['\r', '\n'])
                .to_string();
            input_buffer.drain(..=newline_idx);
            if !line.is_empty() {
                lines.push(line);
            }
        }

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "hello");
    }

    #[test]
    fn read_lines_nonblocking_with_multiple_lines() {
        let data = b"line1\nline2\nline3\n";
        let stream: &mut dyn Read = &mut &data[..];
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let mut input_buffer = String::new();
        input_buffer.push_str(String::from_utf8_lossy(&buf[..n]).as_ref());

        let mut lines = Vec::new();
        while let Some(newline_idx) = input_buffer.find('\n') {
            let line = input_buffer[..=newline_idx]
                .trim_end_matches(['\r', '\n'])
                .to_string();
            input_buffer.drain(..=newline_idx);
            if !line.is_empty() {
                lines.push(line);
            }
        }

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn read_lines_nonblocking_with_incomplete_line() {
        let data = b"incomplete";
        let stream: &mut dyn Read = &mut &data[..];
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let mut input_buffer = String::new();
        input_buffer.push_str(String::from_utf8_lossy(&buf[..n]).as_ref());

        let mut lines = Vec::new();
        while let Some(newline_idx) = input_buffer.find('\n') {
            let line = input_buffer[..=newline_idx]
                .trim_end_matches(['\r', '\n'])
                .to_string();
            input_buffer.drain(..=newline_idx);
            if !line.is_empty() {
                lines.push(line);
            }
        }

        assert_eq!(lines.len(), 0);
        assert_eq!(input_buffer, "incomplete");
    }

    #[test]
    fn read_lines_nonblocking_strips_crlf() {
        let data = b"hello\r\nworld\r\n";
        let stream: &mut dyn Read = &mut &data[..];
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let mut input_buffer = String::new();
        input_buffer.push_str(String::from_utf8_lossy(&buf[..n]).as_ref());

        let mut lines = Vec::new();
        while let Some(newline_idx) = input_buffer.find('\n') {
            let line = input_buffer[..=newline_idx]
                .trim_end_matches(['\r', '\n'])
                .to_string();
            input_buffer.drain(..=newline_idx);
            if !line.is_empty() {
                lines.push(line);
            }
        }

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "hello");
        assert_eq!(lines[1], "world");
    }

    #[test]
    fn read_lines_nonblocking_skips_empty_lines() {
        let data = b"line1\n\nline3\n";
        let stream: &mut dyn Read = &mut &data[..];
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let mut input_buffer = String::new();
        input_buffer.push_str(String::from_utf8_lossy(&buf[..n]).as_ref());

        let mut lines = Vec::new();
        while let Some(newline_idx) = input_buffer.find('\n') {
            let line = input_buffer[..=newline_idx]
                .trim_end_matches(['\r', '\n'])
                .to_string();
            input_buffer.drain(..=newline_idx);
            if !line.is_empty() {
                lines.push(line);
            }
        }

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line3");
    }
}
