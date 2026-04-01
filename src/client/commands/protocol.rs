use std::io;

fn quote_net_argument(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn build_login_request(user_name: &str) -> String {
    format!("C100 LOGIN \"{}\"\r\n", quote_net_argument(user_name))
}

pub fn parse_response_code(response: &str) -> io::Result<u16> {
    let header = response
        .split_whitespace()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "empty server response"))?;

    if header.len() != 4 || !header.starts_with('R') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid response header: {}", header),
        ));
    }

    header[1..].parse::<u16>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid response code: {}", header),
        )
    })
}

pub fn extract_uuid_from_body(response: &str) -> io::Result<String> {
    let body = response
        .split_once(' ')
        .map(|(_, rest)| rest.trim())
        .unwrap_or("");

    if body.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing UUID in login response",
        ));
    }

    if let Some(start) = body.find('"') {
        let tail = &body[start + 1..];
        if let Some(end) = tail.find('"') {
            let uuid = &tail[..end];
            if !uuid.is_empty() {
                return Ok(uuid.to_string());
            }
        }
    }

    let uuid = body.split_whitespace().next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing UUID in login response")
    })?;

    Ok(uuid.to_string())
}
