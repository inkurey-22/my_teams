use crate::commands::handlers;
use crate::commands::{CommandDefinition, CommandMap};

pub fn command_registry() -> CommandMap {
    CommandMap::from([
        (
            "HELP",
            CommandDefinition {
                usage: "HELP",
                description: "show help",
                handler: handlers::handle_help,
            },
        ),
        (
            "LOGIN",
            CommandDefinition {
                usage: "LOGIN [\"user_name\"]",
                description: "set the user_name used by client",
                handler: handlers::handle_login,
            },
        ),
        (
            "LOGOUT",
            CommandDefinition {
                usage: "LOGOUT",
                description: "disconnect the client from the server",
                handler: handlers::handle_logout,
            },
        ),
        (
            "USERS",
            CommandDefinition {
                usage: "USERS",
                description: "list all users that exist on the domain",
                handler: handlers::handle_users,
            },
        ),
        (
            "USER",
            CommandDefinition {
                usage: "USER [\"user_uuid\"]",
                description: "get details about the requested user",
                handler: handlers::handle_user,
            },
        ),
        (
            "SEND",
            CommandDefinition {
                usage: "SEND [\"user_uuid\"] [\"message_body\"]",
                description: "send a message to specific user",
                handler: handlers::handle_send,
            },
        ),
        (
            "MESSAGES",
            CommandDefinition {
                usage: "MESSAGES [\"user_uuid\"]",
                description: "list all messages exchanged with the specified user",
                handler: handlers::handle_messages,
            },
        ),
        (
            "SUBSCRIBE",
            CommandDefinition {
                usage: "SUBSCRIBE [\"team_uuid\"]",
                description: "subscribe to team events and sub resources",
                handler: handlers::handle_subscribe,
            },
        ),
        (
            "SUBSCRIBED",
            CommandDefinition {
                usage: "SUBSCRIBED ?[\"team_uuid\"]",
                description: "list all subscribed teams or users subscribed to a team",
                handler: handlers::handle_subscribed,
            },
        ),
        (
            "UNSUBSCRIBE",
            CommandDefinition {
                usage: "UNSUBSCRIBE [\"team_uuid\"]",
                description: "unsubscribe from a team",
                handler: handlers::handle_unsubscribe,
            },
        ),
        (
            "USE",
            CommandDefinition {
                usage: "USE ?[\"team_uuid\"] ?[\"channel_uuid\"] ?[\"thread_uuid\"]",
                description: "set current command context",
                handler: handlers::handle_use,
            },
        ),
        (
            "CREATE",
            CommandDefinition {
                usage: "CREATE",
                description: "based on context, create a sub resource",
                handler: handlers::handle_create,
            },
        ),
        (
            "LIST",
            CommandDefinition {
                usage: "LIST",
                description: "based on context, list sub resources",
                handler: handlers::handle_list,
            },
        ),
        (
            "INFO",
            CommandDefinition {
                usage: "INFO",
                description: "based on context, display resource details",
                handler: handlers::handle_info,
            },
        ),
    ])
}
