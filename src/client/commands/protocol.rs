use std::io;

/// Server-side info notifications understood by the client.
pub enum InfoMessage {
    NewMessage {
        sender_uuid: String,
        message_body: String,
    },
    NewTeam {
        team_uuid: String,
        team_name: String,
        team_description: String,
    },
    UserLoggedIn {
        user_uuid: String,
        user_name: String,
    },
    UserLoggedOut {
        user_uuid: String,
        user_name: String,
    },
    NewChannel {
        team_uuid: String,
        channel_uuid: String,
        channel_name: String,
        channel_description: String,
    },
    NewThread {
        team_uuid: String,
        channel_uuid: String,
        thread_uuid: String,
        user_uuid: String,
        thread_timestamp: i64,
        thread_title: String,
        thread_body: String,
    },
    NewReply {
        team_uuid: String,
        thread_uuid: String,
        user_uuid: String,
        reply_body: String,
    },
}

/// Escape a request argument for the wire protocol.
fn quote_net_argument(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Build a `LOGIN` request line.
pub fn build_login_request(user_name: &str) -> String {
    format!("C100 LOGIN \"{}\"\r\n", quote_net_argument(user_name))
}

/// Build a `LOGOUT` request line.
pub fn build_logout_request() -> String {
    "C100 LOGOUT\r\n".to_string()
}

/// Build a `USERS` request line.
pub fn build_users_request() -> String {
    "C100 USERS\r\n".to_string()
}

/// Build a `USER` request line.
pub fn build_user_request(user_uuid: &str) -> String {
    format!("C100 USER \"{}\"\r\n", quote_net_argument(user_uuid))
}

/// Build a `SEND` request line.
pub fn build_send_request(user_uuid: &str, message_body: &str) -> String {
    format!(
        "C100 SEND \"{}\" \"{}\"\r\n",
        quote_net_argument(user_uuid),
        quote_net_argument(message_body)
    )
}

/// Build a `MESSAGES` request line.
pub fn build_messages_request(user_uuid: &str) -> String {
    format!("C100 MESSAGES \"{}\"\r\n", quote_net_argument(user_uuid))
}

/// Build a `SUBSCRIBE` request line.
pub fn build_subscribe_request(team_uuid: &str) -> String {
    format!("C100 SUBSCRIBE \"{}\"\r\n", quote_net_argument(team_uuid))
}

/// Build a `SUBSCRIBED` request line.
pub fn build_subscribed_request(team_uuid: Option<&str>) -> String {
    match team_uuid {
        Some(team_uuid) => format!("C100 SUBSCRIBED \"{}\"\r\n", quote_net_argument(team_uuid)),
        None => "C100 SUBSCRIBED\r\n".to_string(),
    }
}

/// Build a `UNSUBSCRIBE` request line.
pub fn build_unsubscribe_request(team_uuid: &str) -> String {
    format!("C100 UNSUBSCRIBE \"{}\"\r\n", quote_net_argument(team_uuid))
}

/// Build a `USE` request line.
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

/// Build a `CREATE_TEAM` request line.
pub fn build_create_team_request(team_name: &str, team_description: &str) -> String {
    format!(
        "C100 CREATE_TEAM \"{}\" \"{}\"\r\n",
        quote_net_argument(team_name),
        quote_net_argument(team_description)
    )
}

/// Build a `CREATE_CHAN` request line.
pub fn build_create_channel_request(channel_name: &str, channel_description: &str) -> String {
    format!(
        "C100 CREATE_CHAN \"{}\" \"{}\"\r\n",
        quote_net_argument(channel_name),
        quote_net_argument(channel_description)
    )
}

/// Build a `CREATE_THREAD` request line.
pub fn build_create_thread_request(thread_title: &str, thread_body: &str) -> String {
    format!(
        "C100 CREATE_THREAD \"{}\" \"{}\"\r\n",
        quote_net_argument(thread_title),
        quote_net_argument(thread_body)
    )
}

/// Build a `CREATE_REP` request line.
pub fn build_create_reply_request(comment_body: &str) -> String {
    format!(
        "C100 CREATE_REP \"{}\"\r\n",
        quote_net_argument(comment_body)
    )
}

/// Build a `LIST_TEAMS` request line.
pub fn build_list_teams_request() -> String {
    "C100 LIST_TEAMS\r\n".to_string()
}

/// Build a `LIST_CHANS` request line.
pub fn build_list_channels_request() -> String {
    "C100 LIST_CHANS\r\n".to_string()
}

/// Build a `LIST_THREADS` request line.
pub fn build_list_threads_request() -> String {
    "C100 LIST_THREADS\r\n".to_string()
}

/// Build a `LIST_REPS` request line.
pub fn build_list_replies_request() -> String {
    "C100 LIST_REPS\r\n".to_string()
}

/// Build a `INFO_USER` request line.
pub fn build_info_user_request() -> String {
    "C100 INFO_USER\r\n".to_string()
}

/// Build a `INFO_TEAM` request line.
pub fn build_info_team_request() -> String {
    "C100 INFO_TEAM\r\n".to_string()
}

/// Build a `INFO_CHAN` request line.
pub fn build_info_channel_request() -> String {
    "C100 INFO_CHAN\r\n".to_string()
}

/// Build a `INFO_THREAD` request line.
pub fn build_info_thread_request() -> String {
    "C100 INFO_THREAD\r\n".to_string()
}

/// Extract the numeric response code from a server response line.
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

/// Extract the first token from a response body as a UUID.
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

/// Tokenize a server response body while preserving quoted payloads.
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

/// Parse an asynchronous info message emitted by the server.
pub fn parse_info_message(line: &str) -> io::Result<Option<InfoMessage>> {
    let mut parts = line.trim().splitn(2, char::is_whitespace);
    let header = parts.next().unwrap_or("");
    if header != "I100" {
        return Ok(None);
    }

    let body = parts.next().unwrap_or("");
    let tokens = tokenize_body(body)?;
    let Some(event_name) = tokens.first().map(|value| value.as_str()) else {
        return Ok(None);
    };

    match event_name {
        "NEW_MESSAGE" => {
            if tokens.len() == 2 {
                return Ok(Some(InfoMessage::NewMessage {
                    sender_uuid: String::new(),
                    message_body: tokens[1].clone(),
                }));
            }

            if tokens.len() == 3 {
                return Ok(Some(InfoMessage::NewMessage {
                    sender_uuid: tokens[1].clone(),
                    message_body: tokens[2].clone(),
                }));
            }

            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid NEW_MESSAGE payload",
            ))
        }
        "NEW_TEAM" => {
            if tokens.len() != 4 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid NEW_TEAM payload",
                ));
            }

            Ok(Some(InfoMessage::NewTeam {
                team_uuid: tokens[1].clone(),
                team_name: tokens[2].clone(),
                team_description: tokens[3].clone(),
            }))
        }
        "USER_LOGGED_IN" => {
            if tokens.len() != 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid USER_LOGGED_IN payload",
                ));
            }

            Ok(Some(InfoMessage::UserLoggedIn {
                user_uuid: tokens[1].clone(),
                user_name: tokens[2].clone(),
            }))
        }
        "USER_LOGGED_OUT" => {
            if tokens.len() != 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid USER_LOGGED_OUT payload",
                ));
            }

            Ok(Some(InfoMessage::UserLoggedOut {
                user_uuid: tokens[1].clone(),
                user_name: tokens[2].clone(),
            }))
        }
        "NEW_CHANNEL" => {
            if tokens.len() != 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid NEW_CHANNEL payload",
                ));
            }

            Ok(Some(InfoMessage::NewChannel {
                team_uuid: tokens[1].clone(),
                channel_uuid: tokens[2].clone(),
                channel_name: tokens[3].clone(),
                channel_description: tokens[4].clone(),
            }))
        }
        "NEW_THREAD" => {
            if tokens.len() != 8 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid NEW_THREAD payload",
                ));
            }

            let thread_timestamp = tokens[5].parse::<i64>().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid NEW_THREAD timestamp payload",
                )
            })?;

            Ok(Some(InfoMessage::NewThread {
                team_uuid: tokens[1].clone(),
                channel_uuid: tokens[2].clone(),
                thread_uuid: tokens[3].clone(),
                user_uuid: tokens[4].clone(),
                thread_timestamp,
                thread_title: tokens[6].clone(),
                thread_body: tokens[7].clone(),
            }))
        }
        "NEW_REPLY" => {
            if tokens.len() != 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid NEW_REPLY payload",
                ));
            }

            Ok(Some(InfoMessage::NewReply {
                team_uuid: tokens[1].clone(),
                thread_uuid: tokens[2].clone(),
                user_uuid: tokens[3].clone(),
                reply_body: tokens[4].clone(),
            }))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_info_message, InfoMessage};

    #[test]
    fn parses_user_login_info_message() {
        let parsed = parse_info_message("I100 USER_LOGGED_IN \"uuid-alice\" \"alice\"")
            .expect("login info should parse");

        assert!(matches!(
            parsed,
            Some(InfoMessage::UserLoggedIn {
                user_uuid,
                user_name,
            }) if user_uuid == "uuid-alice" && user_name == "alice"
        ));
    }

    #[test]
    fn parses_user_logout_info_message() {
        let parsed = parse_info_message("I100 USER_LOGGED_OUT \"uuid-alice\" \"alice\"")
            .expect("logout info should parse");

        assert!(matches!(
            parsed,
            Some(InfoMessage::UserLoggedOut {
                user_uuid,
                user_name,
            }) if user_uuid == "uuid-alice" && user_name == "alice"
        ));
    }

    #[test]
    fn parses_team_creation_info_message() {
        let parsed = parse_info_message("I100 NEW_TEAM \"uuid-team\" \"team name\" \"team description\"")
            .expect("team creation info should parse");

        assert!(matches!(
            parsed,
            Some(InfoMessage::NewTeam {
                team_uuid,
                team_name,
                team_description,
            }) if team_uuid == "uuid-team" && team_name == "team name" && team_description == "team description"
        ));
    }
}
