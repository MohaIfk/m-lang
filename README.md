# The M language

---

## Lexical Grammar (Tokenizer)

### Tokens

| **Category**      | **Token Rules / Patterns**                                                                                                                                                             |
| ----------------- |----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Comments**      | Line: `//...`<br><br>  <br><br>Block: `/* ... */` (Non-nested)                                                                                                                         |
| **Whitespace**    | Space, Tab, Newline, CR (Skipped)                                                                                                                                                      |
| **Keywords**      | `fn` `struct` `enum` `let` `var` `const` `import` `extern`<br><br>  <br><br>`if` `else` `while` `for` `return` `break` `continue`<br><br>  <br><br>`null` `sizeof` `as` `true` `false` |
| **Types**         | `void` `bool` `char`<br><br>  <br><br>`i8` `i16` `i32` `i64`<br><br>  <br><br>`u8` `u16` `u32` `u64`<br><br>  <br><br> `usize` `isize` `f32` `f64`                                     |
| **Int Literal**   | Dec: `[0-9]+(_[0-9]+)*`<br><br>  <br><br>Hex: `0x[0-9a-fA-F_]+`<br><br>  <br><br>Bin: `0b[01_]+`                                                                                       |
| **Float Literal** | `[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?`                                                                                                                                                     |
| **String/Char**   | `"`...`"` and `'`...`'` (Allow escapes: `\n`, `\t`, `\r`, `\\`, `\"`, `\'`)                                                                                                            |
| **Operators**     | `->` `==` `!=` `<=` `>=` `&&` `\|\|`                                                                                                                                                   |

## Syntactic Grammar (Parser)

### Program Structure

```ebnf
program ::= { top_level } EOF ;

top_level ::= import_decl | ( [ attributes ] (
                fn_decl
              | struct_decl
              | enum_decl
              | global_decl
              | extern_decl
              )) ;

attributes ::= "@" IDENTIFIER [ "(" arg_list ")" ] ;

import_decl ::= "import" STRING_LITERAL ";" ;

extern_decl ::= "extern" STRING_LITERAL "fn" IDENTIFIER "(" [ param_list ] ")" [ "->" type ] ";" ;
```

### Types

```ebnf
type ::= primitive_type
       | IDENTIFIER
       | "*" type
       | "[" expression "]" type
       | "fn" "(" [ type_list ] ")" [ "->" type ]
       ;

type_list ::= type { "," type } ;

primitive_type ::= "i8" | "i16" | "i32" | "i64"
                 | "u8" | "u16" | "u32" | "u64"
                 | "usize" | "isize"
                 | "f32" | "f64"
                 | "bool" | "char" | "void" ;
```

### Declarations

```ebnf
// fn add(a: i32, b: i32) -> i32 { ... }
fn_decl ::= "fn" IDENTIFIER "(" [ param_list ] ")" [ "->" type ] block ;

param_list ::= param { "," param } ;
param      ::= IDENTIFIER ":" type ;

// struct Point { x: i32; y: i32; }
struct_decl ::= "struct" IDENTIFIER "{" { struct_field } "}" ;
struct_field ::= IDENTIFIER ":" type ";" ;

// enum Color { Red = 0, Blue }
enum_decl ::= "enum" IDENTIFIER "{" enum_item { "," enum_item } [","] "}" ;
enum_item ::= IDENTIFIER [ "=" expression ] ;

// Global vars
global_decl ::= ("const" | "var") IDENTIFIER ":" type [ "=" expression ] ";" ;
```

### Statements

```ebnf
block ::= "{" { statement } "}" ;

statement ::= var_decl_stmt
            | assignment_stmt
            | if_stmt
            | while_stmt
            | for_stmt
            | return_stmt
            | break_stmt
            | continue_stmt
            | expr_stmt
            | block ;

// Clause: "let x: i32 = 0"
var_decl_clause ::= ("let" | "var") IDENTIFIER ":" type [ "=" expression ] ;
// Statement: Clause + Semicolon
var_decl_stmt   ::= var_decl_clause ";" ;

// Clause: "i = i + 1"
assignment_clause ::= expression "=" expression ; 
// Statement: Clause + Semicolon
assignment_stmt   ::= assignment_clause ";" ;

if_stmt ::= "if" expression block [ "else" ( if_stmt | block ) ] ;

while_stmt ::= "while" expression block ;

for_stmt ::= "for" "(" ( var_decl_stmt | assignment_stmt | ";" ) 
                       [ expression ] ";" 
                       [ assignment_clause ] ")" block ;

return_stmt   ::= "return" [ expression ] ";" ;
break_stmt    ::= "break" ";" ;
continue_stmt ::= "continue" ";" ;
expr_stmt     ::= expression ";" ;
```

# Expressions

***Design Choice*: Assignment is NOT an expression here. You cannot do if (a = b). This prevents a whole class of C bugs.**

```ebnf
expression   ::= logical_or ; 

logical_or   ::= logical_and { "||" logical_and } ;
logical_and  ::= equality { "&&" equality } ;
equality     ::= comparison { ("==" | "!=") comparison } ;
comparison   ::= bitwise_or { ("<" | ">" | "<=" | ">=") bitwise_or } ;

bitwise_or   ::= bitwise_xor { "|" bitwise_xor } ;
bitwise_xor  ::= bitwise_and { "^" bitwise_and } ;
bitwise_and  ::= shift { "&" shift } ;

shift        ::= term { ("<<" | ">>") term } ;
term         ::= factor { ("+" | "-") factor } ;
factor       ::= cast { ("*" | "/" | "%") cast } ;

cast         ::= unary [ "as" type ] ; 

unary        ::= "!" unary
               | "-" unary
               | "*" unary
               | "&" unary
               | "sizeof" "(" (type | expression) ")"
               | postfix ;

postfix      ::= primary { postfix_op } ;
postfix_op   ::= "[" expression "]"
               | "(" [ arg_list ] ")"
               | "." IDENTIFIER
               | "->" IDENTIFIER
               ;

primary      ::= INT_LITERAL | FLOAT_LITERAL | STRING_LITERAL | CHAR_LITERAL
               | "true" | "false" | "null"
               | IDENTIFIER
               | "(" expression ")"
               | struct_init ;

struct_init  ::= IDENTIFIER "{" IDENTIFIER ":" expression { "," IDENTIFIER ":" expression } [","] "}" ;

arg_list     ::= expression { "," expression } ;
```