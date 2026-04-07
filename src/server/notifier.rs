use crate::commands::InfoEvent;

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
