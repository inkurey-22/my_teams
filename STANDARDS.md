# Standards

## Function of the standard

blabla something about the function of the standard
(server-client communication, command formats, etc.)

### Definitions

```Client``` - The entity that initiates communication with the server.  

```User``` - The human who interacts with the client application/command line interface (CLI).  

```Server``` - The entity that processes commands sent by clients and sends responses back to them.  


## Client-Server Communication Standards

### Client sent commands to the server

The client sends commands to the server using a specific format.

#### CLI format:
The user can use the command line interface (CLI) to send commands to the server. The CLI format is designed for human readability and ease of use. Each command starts with a forward slash (/) followed by the command name and any necessary arguments. //The command name is case-insensitive, meaning that it can be written in uppercase, lowercase, or a mix of both.// Arguments are preceded by a space character//, and the command must end with a newline character to indicate the end of the command//. If needs be, the user can also use quotation marks to enclose arguments that contain spaces.

#### NET format:
The network (NET) format is the format used for communication between the client and the server. It is designed for efficient parsing and transmission over the network. Each command is represented as a single line of text, with the command name in uppercase followed by any necessary arguments. Arguments are preceded by a space character, and the command must end with a Carriage Return + Line Feed sequence (CRLF) to indicate the end of the command. The arguments are always separated by quotation marks. 

The following symbols are used in the command formats:  
```<SP>``` : space character  
```<CRLF>``` : Carriage Return + Line Feed sequence  

An argument preceded by a question mark (?) is optional, meaning that the command can be executed without providing that argument.

example dummy command:

command_name : description of the command
CLI ```\command_name <SP> [argument_1] <SP> ?[argument_2]```  
NET ```COMMAND_NAME <SP> ["argument_1"] <SP> "[argument_2]" <CRLF>```  
NET ```COMMAND_NAME <SP> ["argument_1"] <CRLF>```  

#### commands:

help : show help  
CLI ```/help```  
NET ```HELP <CRLF>```  

login : set the user_name used by client  
CLI ```/login ["user_name"]```  
NET ```LOGIN <SP> ["user_name"] <CRLF>```  

logout : disconnect the client from the server  
CLI ```/logout```  
NET ```LOGOUT <CRLF>```  

users : list all users that exist on the domain  
CLI ```/users```  
NET ```USERS <CRLF>```  

user : get details about a specific user  
CLI ```/user ["user_uuid"]```  
NET ```USER <SP> ["user_uuid"] <CRLF>```  

send : send a message to a specific user  
CLI ```/send ["user_uuid"] ["message_body"]```  
NET ```SEND <SP> ["user_uuid"] <SP> ["message_body"] <CRLF>```  

messages : list all messages exchanged with a specific user  
CLI ```/messages ["user_uuid"]```  
NET ```MESSAGES <SP> ["user_uuid"] <CRLF>```  

subscribe : subscribe to the events of a team and its sub directories (enable reception of all events from a team)  
CLI ```/subscribe ["team_uuid"]```  
NET ```SUBSCRIBE <SP> ["team_uuid"] <CRLF>```  

subscribed : list all subscribed teams or list all users subscribed to a team  
CLI ```/subscribed ?["team_uuid"]```  
NET ```SUBSCRIBED <SP> ["team_uuid"] <CRLF>```  
NET ```SUBSCRIBED<CRLF>```  

unsubscribe : unsubscribe from a team and its sub directories (disable reception of all events from a team)  
CLI ```/unsubscribe ["team_uuid"]```  
NET ```UNSUBSCRIBE <SP> ["team_uuid"] <CRLF>```  

use : Sets the command context to a team/channel/thread  
CLI ```/use ?["location_uuid"]```  
NET ```USE <SP> ["location_uuid"] <CRLF>```  
NET ```USE<CRLF>```  

create : based on the context, create the sub resource  
CLI ```/create```  
NET ```CREATE <CRLF>```  

list : based on the context, list all the sub resources  
CLI ```/list```  
NET ```LIST <SP> ["location_uuid"] <CRLF>```  

info : based on the context, display details of the current resource  
CLI ```/info```  
NET ```INFO <CRLF>```  

### Server sent data to the client

The server sends data to the client using a specific format.  

blabla something about the format of the data sent by the server  

```NEW_MESSAGE <SP> ["message_uuid"] <SP> ["sender_uuid"] <SP> ["receiver_uuid"] <SP> ["message_body"] <CRLF>```  

BLABLA  