#Adele
A work in progress interface description language (IDL).

#Syntax
#####Tokens

Tokens are defined by regular expressions.

whitespace: `([\n\r\t ]|(#[^\n]*))+` (regex).
identifier: `[a-zA-Z](_?[a-zA-Z0-9])*` (regex). additionally, an identifier can't be a keyword.
keyword: `type|systems|connect|disconnect|dynamic` (regex), or any primitive.
arrow: `->`
sum: `|`
product: `*`
primitive: `integer|bool|float|double|utf8|array|vector` (regex)


#####Grammar
From here on out whitespaces are ignored. Adele's grammar is defined using ISO 14977 EBNF.
```
<grammar goes here :D>
```



Binary Format
=========

Details go here.