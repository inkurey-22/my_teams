use crate::commands::handlers;
use crate::commands::{CommandDefinition, CommandMap};

/// Build the client command registry.
pub fn command_registry() -> CommandMap {
    CommandMap::from([
        (
            "/help",
            CommandDefinition {
                usage: "/help",
                description: "show help",
                handler: handlers::handle_help,
            },
        ),
        (
            "/login",
            CommandDefinition {
                usage: "/login [\"user_name\"]",
                description: "set the user_name used by client",
                handler: handlers::handle_login,
            },
        ),
        (
            "/logout",
            CommandDefinition {
                usage: "/logout",
                description: "disconnect the client from the server",
                handler: handlers::handle_logout,
            },
        ),
        (
            "/users",
            CommandDefinition {
                usage: "/users",
                description: "get the list of all users that exist on the domain",
                handler: handlers::handle_users,
            },
        ),
        (
            "/user",
            CommandDefinition {
                usage: "/user [\"user_uuid\"]",
                description: "get details about the requested user",
                handler: handlers::handle_user,
            },
        ),
        (
            "/send",
            CommandDefinition {
                usage: "/send [\"user_uuid\"] [\"message_body\"]",
                description: "send a message to specific user",
                handler: handlers::handle_send,
            },
        ),
        (
            "/messages",
            CommandDefinition {
                usage: "/messages [\"user_uuid\"]",
                description: "list all messages exchanged with the specified user",
                handler: handlers::handle_messages,
            },
        ),
        (
            "/subscribe",
            CommandDefinition {
                usage: "/subscribe [\"team_uuid\"]",
                description: "subscribe to team events and sub resources",
                handler: handlers::handle_subscribe,
            },
        ),
        (
            "/subscribed",
            CommandDefinition {
                usage: "/subscribed ?[\"team_uuid\"]",
                description: "list all subscribed teams or users subscribed to a team",
                handler: handlers::handle_subscribed,
            },
        ),
        (
            "/unsubscribe",
            CommandDefinition {
                usage: "/unsubscribe [\"team_uuid\"]",
                description: "unsubscribe from a team",
                handler: handlers::handle_unsubscribe,
            },
        ),
        (
            "/use",
            CommandDefinition {
                usage: "/use ?[\"team_uuid\"] ?[\"channel_uuid\"] ?[\"thread_uuid\"]",
                description: "set current command context",
                handler: handlers::handle_use,
            },
        ),
        (
            "/create",
            CommandDefinition {
                usage: "/create",
                description: "based on context, create a sub resource",
                handler: handlers::handle_create,
            },
        ),
        (
            "/list",
            CommandDefinition {
                usage: "/list",
                description: "based on context, list sub resources",
                handler: handlers::handle_list,
            },
        ),
        (
            "/info",
            CommandDefinition {
                usage: "/info",
                description: "based on context, display resource details",
                handler: handlers::handle_info,
            },
        ),
    ])
}
