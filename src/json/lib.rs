use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

pub type JsonObject = BTreeMap<String, JsonValue>;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(JsonObject),
}

#[derive(Debug)]
pub enum JsonIoError {
    Io(io::Error),
    Parse { message: String, position: usize },
    InvalidRootType(&'static str),
}

impl fmt::Display for JsonIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonIoError::Io(err) => write!(f, "I/O error: {err}"),
            JsonIoError::Parse { message, position } => {
                write!(f, "JSON parse error at byte {position}: {message}")
            }
            JsonIoError::InvalidRootType(msg) => write!(f, "Invalid JSON root type: {msg}"),
        }
    }
}

impl std::error::Error for JsonIoError {}

impl From<io::Error> for JsonIoError {
    fn from(value: io::Error) -> Self {
        JsonIoError::Io(value)
    }
}

pub fn read_json_value<P>(path: P) -> Result<JsonValue, JsonIoError>
where
    P: AsRef<Path>,
{
    let content = fs::read_to_string(path).map_err(JsonIoError::from)?;
    parse_json_value(&content)
}

pub fn write_json_value<P>(path: P, value: &JsonValue) -> Result<(), JsonIoError>
where
    P: AsRef<Path>,
{
    fs::write(path, stringify_json_value(value)).map_err(JsonIoError::from)
}

pub fn read_json_text<P>(path: P) -> Result<String, JsonIoError>
where
    P: AsRef<Path>,
{
    fs::read_to_string(path).map_err(JsonIoError::from)
}

pub fn write_json_text<P>(path: P, json: &str) -> Result<(), JsonIoError>
where
    P: AsRef<Path>,
{
    fs::write(path, json).map_err(JsonIoError::from)
}

pub fn parse_json_value(json: &str) -> Result<JsonValue, JsonIoError> {
    let mut parser = Parser::new(json);
    let value = parser.parse_value()?;
    parser.skip_ws();

    if parser.is_eof() {
        Ok(value)
    } else {
        Err(parser.error("trailing characters after valid JSON"))
    }
}

pub fn parse_json_object(json: &str) -> Result<JsonObject, JsonIoError> {
    match parse_json_value(json)? {
        JsonValue::Object(object) => Ok(object),
        _ => Err(JsonIoError::InvalidRootType(
            "expected a JSON object at top level",
        )),
    }
}

pub fn stringify_json_value(value: &JsonValue) -> String {
    let mut out = String::new();
    write_json_value_to_string(value, &mut out);
    out
}

pub fn stringify_json_object(object: &JsonObject) -> Result<String, JsonIoError> {
    Ok(stringify_json_value(&JsonValue::Object(object.clone())))
}

fn write_json_value_to_string(value: &JsonValue, out: &mut String) {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(v) => {
            if *v {
                out.push_str("true");
            } else {
                out.push_str("false");
            }
        }
        JsonValue::Number(n) => {
            if n.is_finite() {
                out.push_str(&n.to_string());
            } else {
                out.push_str("null");
            }
        }
        JsonValue::String(s) => write_json_string(s, out),
        JsonValue::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_json_value_to_string(item, out);
            }
            out.push(']');
        }
        JsonValue::Object(map) => {
            out.push('{');
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_json_string(k, out);
                out.push(':');
                write_json_value_to_string(v, out);
            }
            out.push('}');
        }
    }
}

fn write_json_string(input: &str, out: &mut String) {
    out.push('"');
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if c < '\u{20}' => {
                out.push_str("\\u");
                out.push_str(&format!("{:04X}", c as u32));
            }
            _ => out.push(ch),
        }
    }
    out.push('"');
}

struct Parser<'a> {
    src: &'a [u8],
    idx: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            src: input.as_bytes(),
            idx: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.idx >= self.src.len()
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.idx).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.idx += 1;
        Some(b)
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if matches!(b, b' ' | b'\n' | b'\r' | b'\t') {
                self.idx += 1;
            } else {
                break;
            }
        }
    }

    fn error(&self, message: &str) -> JsonIoError {
        JsonIoError::Parse {
            message: message.to_string(),
            position: self.idx,
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, JsonIoError> {
        self.skip_ws();
        match self.peek() {
            Some(b'n') => self.parse_null(),
            Some(b't') | Some(b'f') => self.parse_bool(),
            Some(b'-') | Some(b'0'..=b'9') => self.parse_number(),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(_) => Err(self.error("unexpected token while parsing JSON value")),
            None => Err(self.error("unexpected end of input while parsing JSON value")),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, JsonIoError> {
        self.expect_bytes(b"null")?;
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, JsonIoError> {
        if self.match_bytes(b"true") {
            Ok(JsonValue::Bool(true))
        } else if self.match_bytes(b"false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(self.error("invalid boolean literal"))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonIoError> {
        let start = self.idx;

        if self.peek() == Some(b'-') {
            self.idx += 1;
        }

        match self.peek() {
            Some(b'0') => {
                self.idx += 1;
            }
            Some(b'1'..=b'9') => {
                self.idx += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.idx += 1;
                }
            }
            _ => return Err(self.error("invalid number format")),
        }

        if self.peek() == Some(b'.') {
            self.idx += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("fractional part requires at least one digit"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.idx += 1;
            }
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.idx += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.idx += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("exponent requires at least one digit"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.idx += 1;
            }
        }

        let s = std::str::from_utf8(&self.src[start..self.idx])
            .map_err(|_| self.error("invalid utf-8 in number token"))?;
        let n = s
            .parse::<f64>()
            .map_err(|_| self.error("number out of range or malformed"))?;
        Ok(JsonValue::Number(n))
    }

    fn parse_string(&mut self) -> Result<String, JsonIoError> {
        if self.next() != Some(b'"') {
            return Err(self.error("expected opening quote for string"));
        }

        let mut result = String::new();
        loop {
            let byte = self
                .next()
                .ok_or_else(|| self.error("unterminated string literal"))?;

            match byte {
                b'"' => return Ok(result),
                b'\\' => {
                    let escaped = self
                        .next()
                        .ok_or_else(|| self.error("incomplete escape sequence"))?;
                    match escaped {
                        b'"' => result.push('"'),
                        b'\\' => result.push('\\'),
                        b'/' => result.push('/'),
                        b'b' => result.push('\u{08}'),
                        b'f' => result.push('\u{0C}'),
                        b'n' => result.push('\n'),
                        b'r' => result.push('\r'),
                        b't' => result.push('\t'),
                        b'u' => {
                            let codepoint = self.parse_u16_hex()?;
                            if let Some(ch) = char::from_u32(codepoint as u32) {
                                result.push(ch);
                            } else {
                                return Err(self.error("invalid unicode escape code point"));
                            }
                        }
                        _ => return Err(self.error("invalid escape sequence")),
                    }
                }
                b if b < 0x20 => {
                    return Err(self.error("control characters are not allowed in strings"));
                }
                _ => {
                    let ch = self.decode_next_utf8(byte)?;
                    result.push(ch);
                }
            }
        }
    }

    fn decode_next_utf8(&mut self, first: u8) -> Result<char, JsonIoError> {
        let width = utf8_char_width(first);
        if width == 0 {
            return Err(self.error("invalid utf-8 leading byte in string"));
        }
        if width == 1 {
            return Ok(first as char);
        }

        let mut bytes = vec![first];
        for _ in 1..width {
            let b = self
                .next()
                .ok_or_else(|| self.error("truncated utf-8 sequence in string"))?;
            bytes.push(b);
        }

        let s = std::str::from_utf8(&bytes).map_err(|_| self.error("invalid utf-8 in string"))?;
        s.chars()
            .next()
            .ok_or_else(|| self.error("empty utf-8 sequence"))
    }

    fn parse_u16_hex(&mut self) -> Result<u16, JsonIoError> {
        let mut value: u16 = 0;
        for _ in 0..4 {
            let b = self
                .next()
                .ok_or_else(|| self.error("incomplete unicode escape"))?;
            value = (value << 4)
                | match b {
                    b'0'..=b'9' => (b - b'0') as u16,
                    b'a'..=b'f' => (b - b'a' + 10) as u16,
                    b'A'..=b'F' => (b - b'A' + 10) as u16,
                    _ => return Err(self.error("invalid hex digit in unicode escape")),
                };
        }
        Ok(value)
    }

    fn parse_array(&mut self) -> Result<JsonValue, JsonIoError> {
        self.expect_byte(b'[')?;
        self.skip_ws();
        let mut items = Vec::new();

        if self.peek() == Some(b']') {
            self.idx += 1;
            return Ok(JsonValue::Array(items));
        }

        loop {
            items.push(self.parse_value()?);
            self.skip_ws();

            match self.next() {
                Some(b',') => {
                    self.skip_ws();
                }
                Some(b']') => return Ok(JsonValue::Array(items)),
                _ => return Err(self.error("expected ',' or ']' in array")),
            }
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, JsonIoError> {
        self.expect_byte(b'{')?;
        self.skip_ws();
        let mut object = JsonObject::new();

        if self.peek() == Some(b'}') {
            self.idx += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect_byte(b':')?;
            self.skip_ws();

            let value = self.parse_value()?;
            object.insert(key, value);

            self.skip_ws();
            match self.next() {
                Some(b',') => {
                    self.skip_ws();
                }
                Some(b'}') => return Ok(JsonValue::Object(object)),
                _ => return Err(self.error("expected ',' or '}' in object")),
            }
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), JsonIoError> {
        match self.next() {
            Some(b) if b == expected => Ok(()),
            _ => Err(self.error("unexpected token")),
        }
    }

    fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), JsonIoError> {
        if self.match_bytes(expected) {
            Ok(())
        } else {
            Err(self.error("invalid literal"))
        }
    }

    fn match_bytes(&mut self, expected: &[u8]) -> bool {
        if self.src.len() < self.idx + expected.len() {
            return false;
        }
        if &self.src[self.idx..self.idx + expected.len()] == expected {
            self.idx += expected.len();
            true
        } else {
            false
        }
    }
}

fn utf8_char_width(first: u8) -> usize {
    match first {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => 0,
    }
}

//tests
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_json() -> JsonObject {
        JsonObject::from([
            (
                "field string".to_string(),
                JsonValue::String("wawa".to_string()),
            ),
            (
                "field number".to_string(),
                JsonValue::Number(-9876543210.0123),
            ),
            ("field boolean".to_string(), JsonValue::Bool(false)),
        ])
    }

    #[test]
    fn basic_json_object_creation() {
        let obj = create_test_json();
        assert_eq!(obj["field string"], JsonValue::String("wawa".to_string()));
        assert_eq!(obj["field number"], JsonValue::Number(-9876543210.0123));
        assert_eq!(obj["field boolean"], JsonValue::Bool(false));
        assert_eq!(obj.get("nonexistent"), None);
    }

    #[test]
    fn json_object_stringification() {
        let obj = create_test_json();
        assert_eq!(stringify_json_value(&obj["field string"]), r#""wawa""#);
        assert_eq!(
            stringify_json_value(&obj["field number"]),
            "-9876543210.0123"
        );
        assert_eq!(stringify_json_value(&obj["field boolean"]), "false");
    }

    #[test]
    fn json_stringification() {
        let obj = create_test_json();
        let json = stringify_json_object(&obj).unwrap();
        assert_eq!(
            json,
            r#"{"field boolean":false,"field number":-9876543210.0123,"field string":"wawa"}"#
        );
    }

    #[test]
    fn json_file_writing() {
        let mut obj = create_test_json();
        let mut json = stringify_json_object(&obj).unwrap();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test.json");
        write_json_text(test_file.to_str().unwrap(), &json).unwrap();
        json = read_json_text(test_file.to_str().unwrap()).unwrap();
        obj = parse_json_object(&json).unwrap();
        assert_eq!(obj["field string"], JsonValue::String("wawa".to_string()));
        assert_eq!(obj["field number"], JsonValue::Number(-9876543210.0123));
        assert_eq!(obj["field boolean"], JsonValue::Bool(false));
        assert_eq!(obj.get("nonexistent"), None);
    }
}
