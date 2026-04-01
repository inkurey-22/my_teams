use std::io;

pub struct ParsedCommand {
    pub name: String,
    pub args: Vec<String>,
}

fn tokenize(input: &str) -> io::Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quotes => {
                escaped = true;
            }
            '"' => {
                in_quotes = !in_quotes;
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if escaped || in_quotes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unterminated quoted argument",
        ));
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

pub fn parse_request_line(line: &str) -> io::Result<ParsedCommand> {
    let mut parts = line.trim().splitn(2, char::is_whitespace);
    let header = parts
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing request header"))?;

    if header != "C100" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported request header: {}", header),
        ));
    }

    let body = parts
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing command body"))?;

    let tokens = tokenize(body)?;
    if tokens.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing command name",
        ));
    }

    Ok(ParsedCommand {
        name: tokens[0].to_uppercase(),
        args: tokens[1..].to_vec(),
    })
}

fn quote_argument(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn response(code: u16, body: Option<&str>) -> String {
    match body {
        Some(text) => format!("R{:03} {}\r\n", code, text),
        None => format!("R{:03}\r\n", code),
    }
}

pub fn quoted(value: &str) -> String {
    format!("\"{}\"", quote_argument(value))
}
