use crate::commands::InfoEvent;

/// Dispatch info events to matching client sessions.
pub fn dispatch_info_events<Session, UserUuid, SendPayload>(
    sessions: &mut Vec<Session>,
    events: &[InfoEvent],
    mut user_uuid_of: UserUuid,
    mut send_payload: SendPayload,
) where
    UserUuid: FnMut(&Session) -> Option<&str>,
    SendPayload: FnMut(&mut Session, &str) -> bool,
{
    for event in events {
        sessions.retain_mut(|session| {
            if user_uuid_of(session) == Some(event.recipient_user_uuid.as_str()) {
                send_payload(session, &event.payload)
            } else {
                true
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InfoEvent;

    struct TestSession {
        user_uuid: Option<String>,
        received_payloads: Vec<String>,
    }

    #[test]
    fn dispatch_info_events_sends_to_matching_user() {
        let mut sessions = vec![
            TestSession {
                user_uuid: Some("user-alice".to_string()),
                received_payloads: Vec::new(),
            },
            TestSession {
                user_uuid: Some("user-bob".to_string()),
                received_payloads: Vec::new(),
            },
        ];

        let events = vec![InfoEvent {
            recipient_user_uuid: "user-alice".to_string(),
            payload: "message for alice".to_string(),
        }];

        dispatch_info_events(
            &mut sessions,
            &events,
            |session| session.user_uuid.as_deref(),
            |session, payload| {
                session.received_payloads.push(payload.to_string());
                true
            },
        );

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].received_payloads.len(), 1);
        assert_eq!(sessions[0].received_payloads[0], "message for alice");
        assert_eq!(sessions[1].received_payloads.len(), 0);
    }

    #[test]
    fn dispatch_info_events_removes_sessions_when_send_fails() {
        let mut sessions = vec![
            TestSession {
                user_uuid: Some("user-alice".to_string()),
                received_payloads: Vec::new(),
            },
            TestSession {
                user_uuid: Some("user-bob".to_string()),
                received_payloads: Vec::new(),
            },
        ];

        let events = vec![InfoEvent {
            recipient_user_uuid: "user-alice".to_string(),
            payload: "message".to_string(),
        }];

        dispatch_info_events(
            &mut sessions,
            &events,
            |session| session.user_uuid.as_deref(),
            |_, _| false,
        );

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].user_uuid.as_ref().unwrap(), "user-bob");
    }

    #[test]
    fn dispatch_info_events_with_empty_events() {
        let mut sessions = vec![TestSession {
            user_uuid: Some("user-alice".to_string()),
            received_payloads: Vec::new(),
        }];

        dispatch_info_events(
            &mut sessions,
            &[],
            |session| session.user_uuid.as_deref(),
            |session, payload| {
                session.received_payloads.push(payload.to_string());
                true
            },
        );

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].received_payloads.len(), 0);
    }

    #[test]
    fn dispatch_info_events_ignores_no_matching_sessions() {
        let mut sessions = vec![TestSession {
            user_uuid: Some("user-bob".to_string()),
            received_payloads: Vec::new(),
        }];

        let events = vec![InfoEvent {
            recipient_user_uuid: "user-alice".to_string(),
            payload: "message".to_string(),
        }];

        let mut send_called = false;
        dispatch_info_events(
            &mut sessions,
            &events,
            |session| session.user_uuid.as_deref(),
            |_, _| {
                send_called = true;
                true
            },
        );

        assert!(!send_called);
        assert_eq!(sessions.len(), 1);
    }
}
