# Adele

Status: Vaporware.

Adele is an interface description language (IDL) that's minimal and makes useful guarantees. It is motivated by the fact that networking code is tedious to write and is often duplicated across the server and client. Life would be easier if we could define our networking logic in one place and use it to generate code for whatever language we would like.

Adele is oriented around messages. A message has a sender, a name, a type, and a set of valid messages that can follow it. For example imagine a game of Connect Four with separate gui and model logic. The gui's responsibilities would be to ask the server to drop a colored disc in a column, to be able to receive a new board state, and to be able to receive that the game is over. This would be expressed:

```
systems gui model;

type color = red | blue;
type maybecolor = red | blue | neither;
alias place_column = color * integer 0 7;
alias game_state = color * array (array maybecolor 6) 7;
alias game_over_state = maybecolor * game_state;

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

# Code Generation

Each incoming message corresponds to a callback with its argument being the message type, and a return value that is the next outgoing message. Generated code should follow the target language's naming conventions.

# Syntax

##### Tokens

Tokens are defined using regular expressions.

whitespace: `([\n\r\t ]|(#[^\n]*))+` (regex).  
identifier: `[a-zA-Z](_?[a-zA-Z0-9])*` (regex). additionally, an identifier can't be a keyword.  
keyword: `type|alias|of|systems|msg|connect|disconnect` (regex), or any primitive.  
primitive: `integer|bool|float|double|blob|unicode|array|vector` (regex)  
intliteral: `[0-9]+` (regex)  
arrow: `=>`, sum: `|`, product: `*`, eq: `=`, lparen: `(`, rparen: `)`, terminate: `;`  


##### Grammar
Here's the grammar in ISO 14977 EBNF where whitespace is ignored and tokens are in capital letters.
```
adele = systemsdecl, {tydecl}, {messagedecl}, graph;

systemsdecl = SYSTEMS, IDENTIFIER, IDENTIFIER, TERMINATE;

tydecl = TYPE, IDENTIFIER, EQ, tybind
       | ALIAS, IDENTIFIER, EQ, tyexpr;
tybind = bindterm, { SUM, bindterm };
bindterm = IDENTIFIER, [OF, tyexpr];
tyexpr = tyexpr PRODUCT tyexpr
       | LPAREN tyexpr RPAREN
       | IDENTIFIER {tyone};
tyone = IDENTIFIER
      | LPAREN tyexpr RPAREN
      | INTLITERAL;

messagedecl = MSG, IDENTIFIER, IDENTIFIER, tyone, TERMINATE;

graph = graphline {graphline};
graphline = graphident, ARROW, graphident, {ARROW, graphident}, TERMINATE;
graphident = IDENTIFIER | CONNECT | DISCONNECT;
```

Message Encoding
=========

Let's start with a couple definitions:  
`intbits(x, y) -> binarystring`:  a function that takes a nonnegative integer x and converts it to a y-bit binary string
`⊕`: an operator that concatenates two binary strings.

##### Message prefix

The message prefix specifies which node in the graph to transition to.

Let n<sub>0</sub>, n<sub>1</sub>, .., n<sub>k-1</sub> be the current node's neighbors sorted by name, alphabetically ascending. The transition is specified as a 0-based index into those neighbors. For example in a graph with `a => b` and `a => c`, and current node `a`, a transition 0 would mean to go to node `b` and transition 1 would mean to go to node `c`. When encoded as a binary string the transition `i` becomes intbits(i, ⌈log<sub>2</sub> k⌉). This binary string is then concatenated with the message payload.

##### Message payload

The message payload holds the data that is associated with the message. 

value: type | binary encoding | restrictions
--- | --- | ---
false : **bool** | '0' | none
true : **bool** | '1' | none
v : **integer a b** | intbits(v - c + 1, ⌈log<sub>2</sub> c⌉) | a <= v <= b, c = b - a + 1
v : **float** |	IEEE754float(v) | none
v : **double** | IEEE754double(v) | none
size, v : **blob** |	intbits(size, 64) ⊕ rawbits(v) | size < 2<sup>64</sup>
size, v : **unicode** | intbits(size, 64) ⊕ utf8(v) | size < 2<sup>64</sup>
[v<sub>1</sub>, ..., v<sub>n</sub>]: **array t n** | encode(v<sub>1</sub>: t) ⊕ ... ⊕ encode(v<sub>n</sub>: t) | none
size, [v<sub>1</sub>, ..., v<sub>size</sub>]: **vector t** | intbits(size, 64) ⊕ encode(v<sub>1</sub>: t) ⊕ ... ⊕ encode(v<sub>size</sub>: t) | size < 2<sup>64</sup>
v<sub>1</sub>, ..., v<sub>n</sub> : **t<sub>1</sub> \* ... \* t<sub>n</sub>** | encode(v<sub>1</sub>: t) ⊕ ... ⊕ encode(v<sub>n</sub>: t) | none
tag, v: **L<sub>0</sub> t<sub>0</sub> &#124; ... &#124; L<sub>n-1</sub> t<sub>n-1</sub>** | intbits(tag, ⌈log<sub>2</sub> n⌉) ⊕ encode(v: t<sub>tag</sub>) | 0 <= tag < n

##### Bitstring packing

The resulting bitstring needs to be packed into a sequence of bytes. So we define a bitstring written left to right as placed into bytes from low address to high address, and within each byte, from low order bit to high order bit. For example consider a value of type `(integer 0 127) * (array bool 5)`. The integer would be 7 bits `IIIIIII` and the array would be 5 `AAAAA` and in memory it would look like

byte offset| bits
--- | ---
0 |   `AIIIIIII`
1 |   `0000AAAA`

In the last byte, any unused bits are set to 0.
