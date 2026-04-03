use crate::protocol::quoted;

#[derive(Clone)]
pub struct PrivateMessageDispatch {
    pub sender_uuid: String,
    pub receiver_uuid: String,
    pub message_body: String,
}

pub struct ProcessResult {
    pub reply: String,
    pub private_message: Option<PrivateMessageDispatch>,
}

pub fn collect_private_message_dispatch(
    command_name: &str,
    reply: &str,
    sender_uuid: Option<&str>,
    args: &[String],
) -> Option<PrivateMessageDispatch> {
    if command_name != "SEND" || !reply.starts_with("R200") {
        return None;
    }

    match (sender_uuid, args.first(), args.get(1)) {
        (Some(sender_uuid), Some(receiver_uuid), Some(message_body)) => {
            Some(PrivateMessageDispatch {
                sender_uuid: sender_uuid.to_string(),
                receiver_uuid: receiver_uuid.clone(),
                message_body: message_body.clone(),
            })
        }
        _ => None,
    }
}

pub fn build_private_message_info_payload(sender_uuid: &str, message_body: &str) -> String {
    format!(
        "I100 NEW_MESSAGE {} {}\r\n",
        quoted(sender_uuid),
        quoted(message_body)
    )
}
