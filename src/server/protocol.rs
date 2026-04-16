use std::io;

/// Parsed client request line.
pub struct ParsedCommand {
    /// Uppercase command name.
    pub name: String,
    /// Remaining quoted or plain arguments.
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

/// Parse a client request line into a command name and argument list.
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

/// Format a server response line.
pub fn response(code: u16, body: Option<&str>) -> String {
    match body {
        Some(text) => format!("R{:03} {}\r\n", code, text),
        None => format!("R{:03}\r\n", code),
    }
}

/// Quote and escape a response argument.
pub fn quoted(value: &str) -> String {
    format!("\"{}\"", quote_argument(value))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn parse_request_line_valid_command() {
        let line = "C100 LOGIN \"alice\"";
        let result = parse_request_line(line).unwrap();
        assert_eq!(result.name, "LOGIN");
        assert_eq!(result.args, vec!["alice"]);
    }

    #[test]
    fn parse_request_line_command_without_args() {
        let line = "C100 USERS";
        let result = parse_request_line(line).unwrap();
        assert_eq!(result.name, "USERS");
        assert!(result.args.is_empty());
    }

    #[test]
    fn parse_request_line_command_with_multiple_args() {
        let line = "C100 SEND \"uuid-bob\" \"hello world\"";
        let result = parse_request_line(line).unwrap();
        assert_eq!(result.name, "SEND");
        assert_eq!(result.args.len(), 2);
        assert_eq!(result.args[0], "uuid-bob");
        assert_eq!(result.args[1], "hello world");
    }

    #[test]
    fn parse_request_line_lowercase_becomes_uppercase() {
        let line = "C100 login \"alice\"";
        let result = parse_request_line(line).unwrap();
        assert_eq!(result.name, "LOGIN");
    }

    #[test]
    fn parse_request_line_invalid_header() {
        let line = "C101 LOGIN \"alice\"";
        let result = parse_request_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn parse_request_line_missing_header() {
        let line = "LOGIN \"alice\"";
        let result = parse_request_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn parse_request_line_missing_body() {
        let line = "C100";
        let result = parse_request_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn parse_request_line_empty_body_is_error() {
        let line = "C100 ";
        let result = parse_request_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn response_with_body() {
        let result = response(200, Some("OK"));
        assert_eq!(result, "R200 OK\r\n");
    }

    #[test]
    fn response_without_body() {
        let result = response(404, None);
        assert_eq!(result, "R404\r\n");
    }

    #[test]
    fn response_formats_code_with_leading_zeros() {
        let result = response(1, Some("Code"));
        assert_eq!(result, "R001 Code\r\n");
    }

    #[test]
    fn quoted_wraps_string_in_quotes() {
        let result = quoted("hello");
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn quoted_escapes_internal_quotes() {
        let result = quoted("hello\"world");
        assert_eq!(result, "\"hello\\\"world\"");
    }

    #[test]
    fn quoted_escapes_backslashes() {
        let result = quoted("path\\to\\file");
        assert_eq!(result, "\"path\\\\to\\\\file\"");
    }

    fn tokenize_simple(input: &str) -> io::Result<Vec<String>> {
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

    #[test]
    fn tokenize_plain_words() {
        let result = tokenize_simple("foo bar baz").unwrap();
        assert_eq!(result, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn tokenize_quoted_string() {
        let result = tokenize_simple("\"hello world\"").unwrap();
        assert_eq!(result, vec!["hello world"]);
    }

    #[test]
    fn tokenize_mixed_quoted_and_plain() {
        let result = tokenize_simple("foo \"bar baz\" qux").unwrap();
        assert_eq!(result, vec!["foo", "bar baz", "qux"]);
    }

    #[test]
    fn tokenize_escaped_quote_inside_string() {
        let result = tokenize_simple("\"hello\\\"world\"").unwrap();
        assert_eq!(result, vec!["hello\"world"]);
    }

    #[test]
    fn tokenize_escaped_backslash() {
        // Test that backslashes in filenames are preserved
        // Using a path without special escape sequences
        let result = tokenize_simple("\"path\\\\ escaped\"").unwrap();
        assert_eq!(result, vec!["path\\ escaped"]);
    }

    #[test]
    fn tokenize_unterminated_quote_is_error() {
        let result = tokenize_simple("\"unterminated");
        assert!(result.is_err());
    }

    #[test]
    fn tokenize_unterminated_escape_is_error() {
        let result = tokenize_simple("\"text\\");
        assert!(result.is_err());
    }

    #[test]
    fn tokenize_empty_string() {
        let result = tokenize_simple("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn tokenize_only_whitespace() {
        let result = tokenize_simple("   \t  ").unwrap();
        assert!(result.is_empty());
    }
}
