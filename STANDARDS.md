# **RFC 9992: Pouler Communication Protocol (Meunier)**

**Status:** Informational  
**Category:** Internet Protocol  
**Published:** April 2026 

---

## Abstract

This document defines a communication protocol designed for use over the Internet Protocol, enabling users to send messages to individual recipients or groups of users simultaneously. The protocol is implemented in a client-server architecture utilizing bilateral communication channels where clients transmit commands to servers and receive corresponding responses. This specification outlines message formats between clients and servers, supported command sets, expected server responses, and return codes for error handling.

---

## 1. Introduction

The Pouler Communication Protocol (Meunier) provides a standardized framework for client-server messaging over IP networks. It enables:
- Individual user communication via direct messages
- Group communication through team/channel/thread structures  
- Event-driven notifications to subscribed users
- Hierarchical resource management (teams, channels, threads, replies)

The protocol operates in two primary modes:
1. **Command Mode**: Client initiates commands, server processes and responds
2. **Notification Mode**: Server proactively sends information to clients based on events

---

## 2. Definitions

| Term | Description |
|------|-------------|
| `Client` | The entity that initiates communication with the server |
| `User` | The human who interacts with the client application/CLI |
| `Server` | The entity that processes commands and sends responses |
| `Message` | Information sent between client and server (data, commands, or responses) |

### 2.1 Symbol Definitions

```
<SP>   : Space character (ASCII 32)
<CRLF> : Carriage Return + Line Feed sequence (ASCII 13 + 10)
```

---

## 3. Message Format

A message consists of a **header** and **body**, separated by a space separator, terminated with `<CRLF>`.

### 3.1 Header Structure

The header begins with an uppercase letter indicating the message type followed by a three-digit numeric code:

| Type | Description |
|------|-------------|
| `C` | Command (client → server) |
| `R` | Response (server → client) |
| `I` | Info/Notification (server → client) |

### 3.2 Return Codes

The following return codes are used in response messages:

```
100 : command (client sent command to server)
200 : command ok
401 : unauthorized (user not logged in)
404 : not found (command/resource missing)
409 : resource already exists (resource conflict)
```

### 3.3 Message Examples

| Scenario | Format |
|----------|--------|
| Help command | `C100 HELP` |
| Response to help | `R200 "help string"` |
| Server notification | `I100 NEW_MESSAGE //blabla` |

---

## 4. Client Commands

### 4.1 CLI Format Specification

Commands are entered via a Command Line Interface (CLI) with the following format:

```
/command_name [SP] "[argument_1]" [SP] ?"[argument_2]"
```

**Rules:**
- Command names are case-insensitive
- Arguments are space-separated and enclosed in quotation marks
- Optional arguments are prefixed with `?`
- Commands must end with a newline character

### 4.2 Network Format Specification

The network format uses uppercase commands with quoted arguments:

```
C100 <SP> COMMAND_NAME [SP] "[argument_1]" [SP] "[argument_2]" <CRLF>
```

---

## 5. Command Reference

### 5.1 Authentication Commands

| CLI | NET Format | Description |
|-----|------------|-------------|
| `/help` | `C100 HELP` | Display help information |
| `/login "[user_name]"` | `C100 LOGIN <SP> "[user_name]"` | Authenticate user with server |
| `/logout` | `C100 LOGOUT` | Disconnect from server |

### 5.2 User Management Commands

| CLI | NET Format | Description |
|-----|------------|-------------|
| `/users` | `C100 USERS` | List all registered users |
| `/user "[uuid]"` | `C100 USER <SP> "[uuid]"` | Retrieve user details by UUID |

### 5.3 Messaging Commands

| CLI | NET Format | Description |
|-----|------------|-------------|
| `/send "[uuid]" "[message]"` | `C100 SEND <SP> "[uuid]" <SP> "[message]"` | Send message to specific user |
| `/messages "[uuid]"` | `C100 MESSAGES <SP> "[uuid]"` | List messages with a user |

### 5.4 Subscription Commands

| CLI | NET Format | Description |
|-----|------------|-------------|
| `/subscribe "[team_uuid]"` | `C100 SUBSCRIBE <SP> "[team_uuid]"` | Subscribe to team events |
| `/subscribed ?"[team_uuid]"` | `C100 SUBSCRIBED [SP] "[team_uuid]"` | List subscribed teams/users |
| `/unsubscribe "[team_uuid]"` | `C100 UNSUBSCRIBE <SP> "[team_uuid]"` | Unsubscribe from team events |

### 5.5 Context Management Commands

| CLI | NET Format | Description |
|-----|------------|-------------|
| `/use ?"[team]" ?"[channel]" ?"[thread]"` | `C100 USE <SP> "[team]" [SP] "[channel]" [SP] "[thread]"` | Set context (team/channel/thread) |

### 5.6 Resource Management Commands

#### 5.6.1 Create Operations

| Context | CLI Format | NET Command | Description |
|---------|------------|-------------|-------------|
| No context | `/create "[team_name]" "[team_description]"` | `C100 CREATE_TEAM <SP> "[name]" <SP> "[desc]"` | Create a team |
| Team only | `/create "[channel_name]" "[channel_description]"` | `C100 CREATE_CHAN <SP> "[name]" <SP> "[desc]"` | Create a channel within a team |
| Team + Channel | `/create "[thread_title]" "[thread_message]"` | `C100 CREATE_THREAD <SP> "[title]" <SP> "[message]"` | Create a thread within a channel |
| Team + Channel + Thread | `/create "[comment_body]"` | `C100 CREATE_REP <SP> "[body]"` | Create a reply within a thread |

#### 5.6.2 List Operations

| Context | CLI Format | NET Command | Description |
|---------|-------------|-------------|-------------|
| No context | `/list` | `C100 LIST_TEAMS` | List all existing teams |
| Team only | `/list` | `C100 LIST_CHANS` | List all existing channels |
| Team + Channel | `/list` | `C100 LIST_THREADS` | List all existing threads |
| Team + Channel + Thread | `/list` | `C100 LIST_REPS` | List all existing replies |

#### 5.6.3 Info Operations

| Context | CLI Format | NET Command | Description |
|---------|-------------|-------------|
| No context | `/info` | `C100 INFO_USER` | Display currently logged-in user details |
| Team only | `/info` | `C100 INFO_TEAM` | Display currently selected team details |
| Team + Channel | `/info` | `C100 INFO_CHAN` | Display currently selected channel details |
| Team + Channel + Thread | `/info` | `C100 INFO_THREAD` | Display currently selected thread detail |

---

## 6. Server Response Messages

The server sends response messages to the client in response to commands sent by the client. The response message header contains the character 'R' followed by a return code indicating the result of command execution and may also contain additional data or information related to the command in the body.

### 6.1 Response Format Structure

```
R<CODE> <SP> "<response_body>" <CRLF>
```

**Example:** `R200 "User 'john' successfully authenticated"`

`RXXX`: any not specified response code.

### 6.2 Response header

#### 6.2.1 Help

| Response | Description |
|---------|-------------|
| `R200` | Help message displayed |
| `RXXX` | Invalid response |

#### 6.2.2 Login

| Response | Description |
|---------|-------------|
| `R200` | User succesfully logged in |
| `R401` | User failed to logged in (wrong username or password) |
| `RXXX` | Invalid response |

#### 6.2.3 Logout

| Response | Description |
|---------|-------------|
| `R200` | User successfully logged out |
| `R401` | Not logged in |
| `RXXX` | Invalid response |

#### 6.2.4 Users

| Response | Description |
|---------|-------------|
| `R200` | Listed users successfully |
| `R401` | Not logged in |
| `RXXX` | Invalid response |

#### 6.2.5 User

| Response | Description |
|---------|-------------|
| `R200` | User information successfully retrieved |
| `R401` | Not logged in |
| `R404` | User not found |
| `RXXX` | Invalid response |

#### 6.2.6 Send

| Response | Description |
|---------|-------------|
| `R200` | Message successfully sent |
| `R401` | Not logged in |
| `R404` | User not found |
| `RXXX` | Invalid response |

#### 6.2.7 Messages

| Response | Description |
|---------|-------------|
| `R200` | Listed messages successfully |
| `R401` | Not logged in |
| `R404` | User not found |
| `RXXX` | Invalid response |

#### 6.2.8 Subscribe

| Response | Description |
|---------|-------------|
| `R200` | Subscription successfully established |
| `R401` | Not logged in |
| `R404` | Team not found |
| `RXXX` | Invalid response |

#### 6.2.9 Subscribed

| Response | Description |
|---------|-------------|
| `R200` | Listed the teams subscribed to or users subsribed to teams |
| `R401` | Not logged in |
| `R404` | Team not found |
| `RXXX` | Invalid response or payload |

#### 6.2.10 Unsubscribe

| Response | Description |
|---------|-------------|
| `R200` | Unsubscription successfully established |
| `R401` | Not logged in |
| `R404` | Team not found |
| `RXXX` | Invalid response or payload |

#### 6.2.11 Use

| Response | Description |
|---------|-------------|
| `R200` | Succesfully updated context |
| `R404` | Resource not found |

#### 6.2.12 Create

##### 6.2.12.1 No context

| Response | Description |
|---------|-------------|
| `R200` | Team created successfully |
| `R401` | Not logged in |
| `R409` | Team already exist |
| `RXXX` | Invalid response |

##### 6.2.12.2 Team only

| Response | Description |
|---------|-------------|
| `R200` | Channel created successfully |
| `R401` | Not logged in |
| `R404` | Team not found |
| `R409` | Channel already exist |
| `RXXX` | Invalid response |

##### 6.2.12.3 Team + channel

| Response | Description |
|---------|-------------|
| `R200` | Thread created successfully |
| `R401` | Not logged in |
| `R404` | Team or channel not found |
| `R409` | Thread already exist |
| `RXXX` | Invalid response |

##### 6.2.12.4 Team + channel + thread

| Response | Description |
|---------|-------------|
| `R200` | Response created successfully |
| `R401` | Not logged in |
| `R404` | Team, channel or thread not found |
| `RXXX` | Invalid response |

#### 6.2.13 List

##### 6.2.13.1 No context

| Response | Description |
|---------|-------------|
| `R200` | Listed teams |
| `R401` | Not logged in |
| `RXXX` | Invalid response |

##### 6.2.13.2 Team

| Response | Description |
|---------|-------------|
| `R200` | Listed channels in team |
| `R401` | Not logged in |
| `R404` | Team not found |
| `RXXX` | Invalid response |

##### 6.2.13.3 Team + channel

| Response | Description |
|---------|-------------|
| `R200` | Listed threads in channel |
| `R401` | Not logged in |
| `R404` | Team or channel not found |
| `RXXX` | Invalid response |

##### 6.2.13.4 Team + channel + thread

| Response | Description |
|---------|-------------|
| `R200` | Listed responses in thread |
| `R401` | Not logged in |
| `R404` | Team, channel or thread not found |
| `RXXX` | Invalid response |

#### 6.2.14 Info

##### 6.2.14.1 No context

| Response | Description |
|---------|-------------|
| `R200` | Current user information |
| `R401` | Not logged in |
| `RXXX` | Invalid response |

##### 6.2.14.2 Team

| Response | Description |
|---------|-------------|
| `R200` | Team information |
| `R401` | Not logged in |
| `R404` | Team not found |
| `RXXX` | Invalid response |

##### 6.2.14.3 Team + channel

| Response | Description |
|---------|-------------|
| `R200` | Channel information |
| `R401` | Not logged in |
| `R404` | Team or channel not found |
| `RXXX` | Invalid response |

##### 6.2.14.4 Team + channel + thread

| Response | Description |
|---------|-------------|
| `R200` | Thread information |
| `R401` | Not logged in |
| `R404` | Team, channel, or thread not found |
| `RXXX` | Invalid response |

---

## 7. Server Information Messages

The server must send non-response messages to the client in the form of info messages. These messages are initiated by an event occurring on the server side, such as a new message being sent to a user or a change in the state of a resource. The info message header contains the character 'I' followed by a code indicating the type of information being sent and may also contain additional data or information related to the event in the body.

### 7.1 Info Message Types

| Type | Format | Description |
|------|--------|-------------|
| `I100` | `I100 NEW_MESSAGE <SP> "[message_body]" <CRLF>` | A new message has been sent to the user |
| `I200` | `I200 TEAM_CREATED <SP> "[team_uuid]" <SP> "[team_name]" <CRLF>` | A team was created |
| `I300` | `I300 CHANNEL_CREATED <SP> "[channel_uuid]" <SP> "[channel_name]" <CRLF>` | A channel was created |
| `I400` | `I400 THREAD_CREATED <SP> "[thread_uuid]" <SP> "[thread_title]" <CRLF>` | A thread was created |
| `I500` | `I500 REPLY_CREATED <SP> "[reply_uuid]" <SP> "[comment_body]" <CRLF>` | A reply was created |

### 7.2 Global Info Messages

| Format | Description |
|------|-------------|
| `I100 USER_LOGGED_IN <SP> "[user_uuid]" <SP> "[user_name]" <CRLF>` | A user has logged in and every connected user should receive the notification. |
| `I100 USER_LOGGED_OUT <SP> "[user_uuid]" <SP> "[user_name]" <CRLF>` | a user has logged out and every connected user should receive the notification. |

---

## 8. References

### 8.1 Normative References

[1]  **RFC 959** - File Transfer Protocol  
    https://www.rfc-editor.org/rfc/rfc959.txt  

[2]  **RFC 822** - Internet Message Format  
    https://www.rfc-editor.org/rfc/rfc822.txt  

### 8.2 Informative References

[3]  **RFC 791** - Internet Protocol Version 4 (IPv4)  
    https://www.rfc-editor.org/rfc/rfc791.txt  

[4]  **RFC 2616** - Hypertext Transfer Protocol -- HTTP/1.1  
    https://www.rfc-editor.org/rfc/rfc2616.txt  

---

## 9. Author Information

| Name | Organization |
|------|-------------|
| Meunier Team | Pouler Communications |