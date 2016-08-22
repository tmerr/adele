#Adele

Status: Everything here is tentative, I'm working out ideas before the implementation.

Adele is an interface description language (IDL) that's minimal and makes strong guarantees. It is motivated by the fact that networking code is tedious to write and is often duplicated across the server and client. Life would be easier if we could define our networking logic in one place and use it to generate code for whatever language we would like.

Adele is oriented around messages. A message has a sender, a name, a type, and a set of valid messages that can follow it. For example imagine a game of Connect Four with separate gui and model logic. The gui's responsibilities would be to ask the server to drop a colored disc in a column, to be able to receive a new board state, and to be able to receive that someone won. This would be expressed:

```
systems gui model;

type color = red | blue;
type maybecolor = red | blue | neither;
type place_column = color * integer 0 7;
type game_state = array (array maybecolor 6) 7;
type game_over_state = maybecolor * board_state;

msg gui place_disc place_column;
msg model update_board game_state;
msg model announce_game_over game_over_state;

connect => place_disc => update_board => place_disc;
           place_disc => announce_game_over => disconnect;
```

The first line does nothing more than name the systems that will be communicating. Then the message types are defined. This is followed by the messages themselves, and what messages can follow one another.

The types use a familiar notation if you're coming from a functional programming background. `type color = red | blue` represents a sum type because it is the *sum* of possibilities red and blue. On the other hand `type place_column = color * integer 0 7` is a product type because the possibilities belong to the *cartesian product* of the set of possible colors and integers 0 through 7. In simpler terms a product type corresponds to a tuple.

The portion of the code that says `connect => ...` describes what messages can follow one another. This information gives us one more way to verify the correctness of an implementation. The generated code can leverage static type checking in ML-like languages, or dynamic checks in Python, to guarantee that a valid sequence of messages are sent during each session.

Because the `connect => ...` portion of code conceptually represents a graph I have created a [visual graph editor](https://github.com/tmerr/adele-gui) to make it easier to edit and understand it.

#Code Generation

Each incoming message corresponds to a callback with its argument being the message type, and a return value that is the next outgoing message.

Generated code should follow the target language's naming conventions.

#Syntax

#####Tokens

Tokens are defined using regular expressions.

whitespace: `([\n\r\t ]|(#[^\n]*))+` (regex).  
identifier: `[a-zA-Z](_?[a-zA-Z0-9])*` (regex). additionally, an identifier can't be a keyword.  
keyword: `type|systems|msg|connect|disconnect|dynamic` (regex), or any primitive.  
primitive: `integer|bool|float|double|blob|utf8|array|vector` (regex)  
intliteral: `[0-9]+` (regex)  
arrow: `=>`, sum: `|`, product: `*`, eq: `=`, lparen: `(`, rparen: `)`, terminate: `;`  


#####Grammar
Here's the grammar in ISO 14977 EBNF where whitespace is ignored and tokens are in capital letters.
```
adele = typedecls, systemsdecl, graph;
typedecls = typedecl, {typedecl};

typedecl = [DYNAMIC] IDENTIFIER, EQ, tyroot, TERMINATE;
tyroot = tycon {tynested}
       | typroduct
       | tysum;
typroduct = tynested, PRODUCT, tynested;
tysum = IDENTIFIER, tynested, SUM, IDENTIFER, tynested;
tynested = tycon
         | LPAREN tyroot RPAREN;
tycon = PRIMITIVE
      | INTLITERAL
      | IDENTIFIER;

systemsdecl = SYSTEMS, IDENTIFIER, IDENTIFIER, TERMINATE;

messagedecls = messagedecl {messagedecl};
messagedecl = MSG, IDENTIFIER, IDENTIFIER, IDENTIFIER, TERMINATE;

graph = graphline {graphline};
graphline = graphident, ARROW, graphident, {ARROW, graphident}, TERMINATE;
graphident = IDENTIFIER | CONNECT | DISCONNECT;
```



Message Encoding
=========

More to come.