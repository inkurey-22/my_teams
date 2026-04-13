use std::io;

fn quote_net_argument(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn build_login_request(user_name: &str) -> String {
    format!("C100 LOGIN \"{}\"\r\n", quote_net_argument(user_name))
}

pub fn build_logout_request() -> String {
    "C100 LOGOUT\r\n".to_string()
}

pub fn build_users_request() -> String {
    "C100 USERS\r\n".to_string()
}

pub fn build_user_request(user_uuid: &str) -> String {
    format!("C100 USER \"{}\"\r\n", quote_net_argument(user_uuid))
}

pub fn build_send_request(user_uuid: &str, message_body: &str) -> String {
    format!(
        "C100 SEND \"{}\" \"{}\"\r\n",
        quote_net_argument(user_uuid),
        quote_net_argument(message_body)
    )
}

pub fn build_messages_request(user_uuid: &str) -> String {
    format!("C100 MESSAGES \"{}\"\r\n", quote_net_argument(user_uuid))
}

pub fn build_use_request(args: &[String]) -> String {
    if args.is_empty() {
        return "C100 USE\r\n".to_string();
    }

    let quoted_args = args
        .iter()
        .map(|arg| format!("\"{}\"", quote_net_argument(arg)))
        .collect::<Vec<_>>()
        .join(" ");

    format!("C100 USE {}\r\n", quoted_args)
}

pub fn build_create_team_request(team_name: &str, team_description: &str) -> String {
    format!(
        "C100 CREATE_TEAM \"{}\" \"{}\"\r\n",
        quote_net_argument(team_name),
        quote_net_argument(team_description)
    )
}

pub fn build_create_channel_request(channel_name: &str, channel_description: &str) -> String {
    format!(
        "C100 CREATE_CHAN \"{}\" \"{}\"\r\n",
        quote_net_argument(channel_name),
        quote_net_argument(channel_description)
    )
}

pub fn build_create_thread_request(thread_title: &str, thread_body: &str) -> String {
    format!(
        "C100 CREATE_THREAD \"{}\" \"{}\"\r\n",
        quote_net_argument(thread_title),
        quote_net_argument(thread_body)
    )
}

pub fn build_create_reply_request(comment_body: &str) -> String {
    format!(
        "C100 CREATE_REP \"{}\"\r\n",
        quote_net_argument(comment_body)
    )
}

pub fn build_list_teams_request() -> String {
    "C100 LIST_TEAMS\r\n".to_string()
}

pub fn build_list_channels_request() -> String {
    "C100 LIST_CHANS\r\n".to_string()
}

pub fn build_list_threads_request() -> String {
    "C100 LIST_THREADS\r\n".to_string()
}

pub fn build_list_replies_request() -> String {
    "C100 LIST_REPS\r\n".to_string()
}

pub fn build_info_user_request() -> String {
    "C100 INFO_USER\r\n".to_string()
}

pub fn build_info_team_request() -> String {
    "C100 INFO_TEAM\r\n".to_string()
}

pub fn build_info_channel_request() -> String {
    "C100 INFO_CHAN\r\n".to_string()
}

pub fn build_info_thread_request() -> String {
    "C100 INFO_THREAD\r\n".to_string()
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
    let tokens = parse_response_tokens(response)?;
    let Some(uuid) = tokens.first() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing UUID in login response",
        ));
    };

    Ok(uuid.to_string())
}

fn tokenize_body(input: &str) -> io::Result<Vec<String>> {
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
            "unterminated info message",
        ));
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

pub fn parse_response_tokens(response: &str) -> io::Result<Vec<String>> {
    let body = response
        .split_once(' ')
        .map(|(_, rest)| rest.trim())
        .unwrap_or("");

    if body.is_empty() {
        return Ok(Vec::new());
    }

    tokenize_body(body)
}

pub fn parse_new_message_info(line: &str) -> io::Result<Option<(String, String)>> {
    let mut parts = line.trim().splitn(2, char::is_whitespace);
    let header = parts.next().unwrap_or("");
    if header != "I100" {
        return Ok(None);
    }

    let body = parts.next().unwrap_or("");
    let tokens = tokenize_body(body)?;
    if tokens.first().map(|t| t.as_str()) != Some("NEW_MESSAGE") {
        return Ok(None);
    }

    if tokens.len() == 2 {
        return Ok(Some((String::new(), tokens[1].clone())));
    }

    if tokens.len() == 3 {
        return Ok(Some((tokens[1].clone(), tokens[2].clone())));
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid NEW_MESSAGE payload",
    ))
}
