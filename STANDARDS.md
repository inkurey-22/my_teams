# RFC 9992: Pouler Communication Protocol (Meunier)

---

## Function of the standard

This standard defines a communication protocol meant to be used over the Internet Protocol allowing users to send messages to each other or defined groups of users at once.
This protocol is designed to be used in a client-server architecture using a bilateral communication channel. where clients send commands to a server and the server processes those commands and sends responses back to the clients. The protocol defines the format of the messages sent between the clients and the server, as well as the commands that clients can send to the server and the expected responses from the server.

---

## Definitions

```Client``` - The entity that initiates communication with the server.

```User``` - The human who interacts with the client application/command line interface (CLI).

```Server``` - The entity that processes commands sent by clients and sends responses back to them.

```Message``` - A piece of information sent from the server to the client or from the client to the server. It can contain data, commands, or responses.

The following symbols are used in the message and command formats:
```<SP>``` : space character
```<CRLF>``` : Carriage Return + Line Feed sequence of characters

---

## Message format

A message is a string of characters comprised of a header and a body. The header and body are separated by a separator (space). The message is terminated by a Carriage Return + Line Feed sequence.

### Message Header format

The message header starts with a uppercase letter indicating the type of message being sent followed by a three characters numeric code.

The list of message types is as follow:
```C``` : a command sent from the client to the server.
```R``` : a response to a command, sent by the server to a client.
```I``` : an info sent by the server to the client.

The list of return codes is as follow:

100 : command (client sent command to the server)
200 : command ok
401 : unauthorized (user not logged in)
403 : forbidden (user is logged in but does not have permission to execute the command)
404 : not found (command not found / resource not found)
500 : internal server error
501 : bad request
502 : gateway timeout //maybe unused
503 : service unavailable //maybe unused

This section describes the return codes that the server sends to the client, it is heavily inspired by http.

here are examples of messages in different situations:
```C100 HELP``` : client sent help command.
```R200 "help string"``` : server sent response to the client sent help command.
```I100 NEW_MESSAGE //blabla``` : server sent notification to the client.

---

## Client sent commands to the server

Each command is initiated by a user written input in the CLI format and then translated in the NET format and sent to the server as a command message.
The client is to display the response message sent by the server.

### CLI format:
The user can use the command line interface (CLI) to send commands to the server. The CLI format is designed for human readability and ease of use. Each command starts with a forward slash (/) followed by the command name and any necessary arguments.
The command name is case-insensitive, meaning that it can be written in uppercase, lowercase, or a mix of both.
Arguments are preceded by a space character, and the command must end with a newline character to indicate the end of the command.
If needs be, the user can also use quotation marks to enclose arguments that contain spaces.
An argument preceded by a question mark (?) is optional, meaning that the command can be executed without providing that argument.

### NET format:

The network (NET) format is the format used for communication between the client and the server. It must respect the message header format defined above.
The body is comprised of the command string as uppercase then arguments each preceeded by a separator (space) if needs be. All arguments are enclosed in quotation marks.
An argument preceded by a question mark (?) is optional, meaning that the command can be executed without providing that argument.

### commands:

example dummy command:

command_name : description of the command
CLI ```/command_name <SP> "[argument_1]" <SP> ?"[argument_2]"```
NET ```C100 <SP> COMMAND_NAME <SP> "[argument_1]" <SP> "[argument_2]" <CRLF>```
NET ```C100 <SP> COMMAND_NAME <SP> "[argument_1]" <CRLF>```

list of commands:

help : show help
CLI ```/help```
NET ```C100 <SP> HELP <CRLF>```

login : set the user_name used by client
CLI ```/login "[user_name]"```
NET ```C100 <SP> LOGIN <SP> "[user_name]" <CRLF>```

logout : disconnect the client from the server
CLI ```/logout```
NET ```C100 <SP> LOGOUT <CRLF>```

users : list all users that exist on the domain
CLI ```/users```
NET ```C100 <SP> USERS <CRLF>```

user : get details about a specific user
CLI ```/user "[user_uuid]"```
NET ```C100 <SP> USER <SP> "[user_uuid]" <CRLF>```

send : send a message to a specific user
CLI ```/send "[user_uuid]" "[message_body]"```
NET ```C100 <SP> SEND <SP> "[user_uuid]" <SP> "[message_body]" <CRLF>```

messages : list all messages exchanged with a specific user
CLI ```/messages "[user_uuid]"```
NET ```C100 <SP> MESSAGES <SP> "[user_uuid]" <CRLF>```

subscribe : subscribe to the events of a team and its sub directories (enable reception of all events from a team)
CLI ```/subscribe "[team_uuid]"```
NET ```C100 <SP> SUBSCRIBE <SP> "[team_uuid]" <CRLF>```

subscribed : list all subscribed teams or list all users subscribed to a team
CLI ```/subscribed ?"[team_uuid]"```
NET ```C100 <SP> SUBSCRIBED <SP> "[team_uuid]" <CRLF>```
NET ```C100 <SP> SUBSCRIBED <CRLF>```

unsubscribe : unsubscribe from a team and its sub directories (disable reception of all events from a team)
CLI ```/unsubscribe "[team_uuid]"```
NET ```C100 <SP> UNSUBSCRIBE <SP> "[team_uuid]" <CRLF>```

use : Sets the command context to a team/channel/thread
CLI ```/use ?"[team_uuid]" ?"[channel_uuid]" ?"[thread_uuid]"```
NET ```C100 <SP> USE <SP> "[team_uuid]" <SP> "[channel_uuid]" <SP> "[thread_uuid]" <CRLF>```
NET ```C100 <SP> USE <SP> "[team_uuid]" <SP> "[channel_uuid]" <CRLF>```
NET ```C100 <SP> USE <SP> "[team_uuid]" <CRLF>```
NET ```C100 <SP> USE <CRLF>```

create : based on the context, create the sub resource
CLI ```/create```
NET : context-dependent (see section below)

list : based on the context, list all the sub resources
CLI ```/list```
NET : context-dependent (see section below)

info : based on the context, display details of the current resource
CLI ```/info```
NET : context-dependent (see section below)

### Context-dependent behavior

#### /create

When the context is not defined (``/use``):
CLI ```/create "[team_name]" "[team_description]"```
NET ```C100 <SP> CREATE_TEAM <SP> "[team_name]" <SP> "[team_description]" <CRLF>```
Meaning: create a new team.

When ``team_uuid`` is defined (``/use "team_uuid"``):
CLI ```/create "[channel_name]" "[channel_description]"```
NET ```C100 <SP> CREATE_CHAN <SP> "[channel_name]" <SP> "[channel_description]" <CRLF>```
Meaning: create a new channel.

When ``team_uuid`` and ``channel_uuid`` are defined (``/use "team_uuid" "channel_uuid"``):
CLI ```/create "[thread_title]" "[thread_message]"```
NET ```C100 <SP> CREATE_THREAD <SP> "[thread_title]" <SP> "[thread_message]" <CRLF>```
Meaning: create a new thread.

When ``team_uuid``, ``channel_uuid`` and ``thread_uuid`` are defined (``/use "team_uuid" "channel_uuid" "thread_uuid"``):
CLI ```/create "[comment_body]"```
NET ```C100 <SP> CREATE_REP <SP> "[comment_body]" <CRLF>```
Meaning: create a new reply.

#### /list

When the context is not defined (``/use``):
CLI ```/list```
NET ```C100 <SP> LIST_TEAMS <CRLF>```
Meaning: list all existing teams.

When ``team_uuid`` is defined (``/use "team_uuid"``):
CLI ```/list```
NET ```C100 <SP> LIST_CHANS <CRLF>```
Meaning: list all existing channels.

When ``team_uuid`` and ``channel_uuid`` are defined (``/use "team_uuid" "channel_uuid"``):
CLI ```/list```
NET ```C100 <SP> LIST_THREADS <CRLF>```
Meaning: list all existing threads.

When ``team_uuid``, ``channel_uuid`` and ``thread_uuid`` are defined (``/use "team_uuid" "channel_uuid" "thread_uuid"``):
CLI ```/list```
NET ```C100 <SP> LIST_REPS <CRLF>```
Meaning: list all existing replies.

#### /info

When the context is not defined (``/use``):
CLI ```/info```
NET ```C100 <SP> INFO_USER <CRLF>```
Meaning: display currently logged-in user details.

When ``team_uuid`` is defined (``/use "team_uuid"``):
CLI ```/info```
NET ```C100 <SP> INFO_TEAM <CRLF>```
Meaning: display currently selected team details.

When ``team_uuid`` and ``channel_uuid`` are defined (``/use "team_uuid" "channel_uuid"``):
CLI ```/info```
NET ```C100 <SP> INFO_CHAN <CRLF>```
Meaning: display currently selected channel details.

When ``team_uuid``, ``channel_uuid`` and ``thread_uuid`` are defined (``/use "team_uuid" "channel_uuid" "thread_uuid"``):
CLI ```/info```
NET ```C100 <SP> INFO_THREAD <CRLF>```
Meaning: display currently selected thread detail.

### Response messages sent by the server to the client

The server sends response messages to the client in response to commands sent by the client. The response message header contains the character 'R' followed by a return code indicating the result of the command execution and may also contain additional data or information related to the command in the body.

---

## Server sent data to the client

The server must send non response messages to the client in the form of info messages. These messages are initiated by an event occurring on the server side, such as a new message being sent to a user or a change in the state of a resource. The info message header contains the character 'I' followed by a code indicating the type of information being sent and may also contain additional data or information related to the event in the body. These messages are sent in the NET format defined below.

### NET format:

The network (NET) format is the format used for communication between the server and the client. It must respect the message header format defined above.
The body contains the information related to the event in the form of a string.

### Info message types:

NET```I100 NEW_MESSAGE <SP> "[message_body]" <CRLF>``` : a new message has been sent to the user.

///ECT
