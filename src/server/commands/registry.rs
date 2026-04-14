use crate::commands::handlers;
use crate::commands::{CommandDefinition, CommandMap};

/// Build the server command registry.
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
            "CREATE_TEAM",
            CommandDefinition {
                usage: "CREATE_TEAM [\"team_name\"] [\"team_description\"]",
                description: "create a team in root context",
                handler: handlers::handle_create,
            },
        ),
        (
            "CREATE_CHAN",
            CommandDefinition {
                usage: "CREATE_CHAN [\"channel_name\"] [\"channel_description\"]",
                description: "create a channel in team context",
                handler: handlers::handle_create,
            },
        ),
        (
            "CREATE_THREAD",
            CommandDefinition {
                usage: "CREATE_THREAD [\"thread_title\"] [\"thread_body\"]",
                description: "create a thread in channel context",
                handler: handlers::handle_create,
            },
        ),
        (
            "CREATE_REP",
            CommandDefinition {
                usage: "CREATE_REP [\"reply_body\"]",
                description: "create a reply in thread context",
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
            "LIST_TEAMS",
            CommandDefinition {
                usage: "LIST_TEAMS",
                description: "list teams from root context",
                handler: handlers::handle_list,
            },
        ),
        (
            "LIST_CHANS",
            CommandDefinition {
                usage: "LIST_CHANS",
                description: "list channels from team context",
                handler: handlers::handle_list,
            },
        ),
        (
            "LIST_THREADS",
            CommandDefinition {
                usage: "LIST_THREADS",
                description: "list threads from channel context",
                handler: handlers::handle_list,
            },
        ),
        (
            "LIST_REPS",
            CommandDefinition {
                usage: "LIST_REPS",
                description: "list replies from thread context",
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
        (
            "INFO_USER",
            CommandDefinition {
                usage: "INFO_USER",
                description: "show current user details",
                handler: handlers::handle_info,
            },
        ),
        (
            "INFO_TEAM",
            CommandDefinition {
                usage: "INFO_TEAM",
                description: "show selected team details",
                handler: handlers::handle_info,
            },
        ),
        (
            "INFO_CHAN",
            CommandDefinition {
                usage: "INFO_CHAN",
                description: "show selected channel details",
                handler: handlers::handle_info,
            },
        ),
        (
            "INFO_THREAD",
            CommandDefinition {
                usage: "INFO_THREAD",
                description: "show selected thread details",
                handler: handlers::handle_info,
            },
        ),
    ])
}
