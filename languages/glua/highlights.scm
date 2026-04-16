; Inherit all standard Lua highlights from tree-sitter-lua.
; This file adds GLua-specific overrides on top.

; Keywords
[
  "and"
  "break"
  "do"
  "else"
  "elseif"
  "end"
  "false"
  "for"
  "function"
  "goto"
  "if"
  "in"
  "local"
  "nil"
  "not"
  "or"
  "repeat"
  "return"
  "then"
  "true"
  "until"
  "while"
  "continue"
] @keyword

; Functions
(function_declaration
  name: (identifier) @function)

(function_declaration
  name: (dot_index_expression) @function)

(function_declaration
  name: (method_index_expression) @function)

(local_function_statement
  name: (identifier) @function)

(function_call
  name: (identifier) @function.call)

(function_call
  name: (dot_index_expression
    field: (identifier) @function.call))

(function_call
  name: (method_index_expression
    method: (identifier) @function.call))

; Parameters
(parameters
  name: (identifier) @variable.parameter)

; Types / class names (common GMod globals)
((identifier) @type
 (#match? @type "^[A-Z][A-Z0-9_]+$"))

; Strings
(string) @string
(string_start) @string
(string_end) @string

; String escape sequences
(escape_sequence) @string.escape

; Numbers
(number) @number

; Booleans / nil
(true) @boolean
(false) @boolean
(nil) @constant.builtin

; Operators — standard Lua
[
  "+"  "-"  "*"  "/"  "%"  "^"  "#"
  "&"  "~"  "|"  "<<"  ">>"  "//"
  "=="  "~="  "<"  "<="  ">"  ">="
  "="
  "("  ")"  "{"  "}"  "["  "]"
  "::"
  ";"  ":"  ","  "."  ".."  "..."
] @operator

; GLua C-style operators — these are just tokenized as identifiers or
; punctuation by the Lua grammar when nonstandardSymbol is configured,
; so we highlight them as operators by matching the raw token text.
; glua-ls handles the semantic understanding; we just need them visible.
((identifier) @operator
 (#any-of? @operator "&&" "||" "!=" "!"))

; Comments
(comment) @comment
(hash_bang_line) @comment

; Variables
(identifier) @variable

; Self
((identifier) @variable.special
 (#eq? @variable.special "self"))

; Fields
(dot_index_expression
  field: (identifier) @property)

(bracket_index_expression
  field: (identifier) @property)

; Labels
(label_statement
  (identifier) @label)

(goto_statement
  (identifier) @label)
