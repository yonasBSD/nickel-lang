---
slug: syntax
---

# Nickel Syntax

## Identifiers

Nickel identifiers start with zero or more underscores `_`, followed by an
alphabetic character (`a` to `z` or `A` to `Z`). They are then followed by zero
or more alphanumeric characters (alphabetic characters or digits `0` to `9`),
`_` (underscores), `-` (dashes) or `'` (single quotes). For example,
`___This-isn't_invalid` is a valid identifier.

## Simple values

There are four basic kinds of values in Nickel :

1. numeric values
2. boolean values
3. strings
4. enum tags

### Numeric values

Nickel has support for numbers, positive and negative, with or without a
fractional part. Internally, those numbers are stored as arbitrary precision
rationals, meaning that basic arithmetic operations (addition, subtraction,
division and multiplication) don't incur rounding errors. Numbers are
deserialized as 64-bit floating point numbers, in line with common JSON
implementations.

Exponentiation is supported using the `std.number.pow` function. If the exponent
is exactly representable as an integer between `-2^63` and `2^64 - 1`, the
result is computed exactly. However, raising a number to a non-integral power
can incur rounding errors: both operands will be converted to the nearest 64-bit
floating point numbers, the power is computed as a 64-bit floating point number
as well, and then converted back to an arbitrary precision rational number.

Numbers are serialized as integers whenever possible, that is, when they
fit exactly into a 64-bit signed integer or a 64-bit unsigned integer. They
are serialized as a 64-bit float otherwise. The latter conversion might lose
precision as well, for example when serializing `1/3`.

Number literals can be written in:

- decimal representation
- hexadecimal representation with a leading `0x`
- octal representation with a leading `0o`
- binary representation with a leading `0b`

The decimal representation supports scientific notation and an optional
fractional part delimited by `.`, while other representations only support
integers.

Here are some examples of number literals in Nickel:

```nickel #lines
1
0.543
1.7e217
-3e-3
-1000000
-6.8
0xFF15a
0b001101
0o77012
```

There are some predefined operators for working with numbers:

| Operator | Description                                          | Example       |
|:--------:|:----------------------------------------------------:|:-------------:|
| +        | The addition operator                                | `1 + 2 = 3`   |
| \-       | The subtraction operator                             | `1 - 2 = -1`  |
| *        | The multiplication operator                          | `1 * 2 = 2`   |
| /        | The division operator                                | `1 / 2 = 0.5` |
| %        | The modulo operator (returns the *signed* remainder) | `5 % 3 = 2`   |

> **Remark about the `-` operator:** Since `-` can be used inside an identifier,
> the subtraction operators **needs** to be surrounded by spaces: write `a - b`,
> not `a-b`. `1-2` works as expected, because `1` and `2` aren't identifiers.

Numbers can be compared using the following operators:

| Operator | Description      | Example   |
|:--------:|:----------------:|:---------:|
| ==       | Equal            | `5 == 5`  |
| !=       | Not Equal        | `5 != 4`  |
| <        | Smaller than     | `2 < 3`   |
| >        | Greater than     | `1 > -5`  |
| >=       | Greater or Equal | `1 >= 1`  |
| <=       | Smaller or Equal | `-1 <= 6` |

In the table below, you will find the operators sorted from highest to lowest precedence:

|       Operators      | Associativity | Remark                                         |
|:--------------------:|:-------------:|------------------------------------------------|
|       `( ... )`      |               | parentheses always have the highest precedence |
|          `-`         |               | unary negation (as in `-1`)                    |
|     `*`, `/`, `%`    | left-to-right |                                                |
|       `+`, `-`       | left-to-right | binary addition and subtraction                |
| `<`, `>`, `=<`, `>=` | left-to-right |                                                |
|      `==`, `!=`      | left-to-right |                                                |

### Boolean values

The boolean values in Nickel are denoted `true` and `false`.

Nickel features the classical boolean operators *AND* (&&), *OR* (||) and *NOT*
(!). The *AND* and *OR* operators are lazy in the evaluation of the second
argument: for example, in `exp1 && exp2`, `exp2` is only evaluated if `exp1`
evaluates to `false`.

Here are some examples of boolean operators in Nickel:

```nickel #repl
> true && false
false

> false || true
true

> ! true
false
```

### Strings

Nickel can work with sequences of characters, or strings. Strings are enclosed
by `" ... "` for a single line string or by `m%" ... "%` for a multiline string.
They can be concatenated with the operator `++`. Strings must be UTF-8 valid.
In fact, as far as at all practicable, Nickel treats strings as sequences of
Unicode extended grapheme clusters and refuses to break them apart.

The string interpolation syntax is
`"%{ < expression that evaluates to a string > }"`. You can interpolate any
primitive value that can be converted to a string, that is numbers, booleans,
enum tags, strings and the `null` value (see `std.string.Stringable`).

Here are some examples of string handling in Nickel:

```nickel #repl
> "Hello, World!"
"Hello, World!"

> m%"Well, if this isn't a multiline string?
  Yes it is, indeed it is"%
"Well, if this isn't a multiline string?\n  Yes it is, indeed it is"

> "Hello" ++ "World"
"HelloWorld"

> let h = "Hello" in "%{h} World"
"Hello World"

> let n = 5 in "The number %{n}."
"The number 5."
```

Multiline strings are useful for writing indented lines. The first and last
lines are ignored if they are empty or contain only spaces. Indentation that is
present on all lines of the string is stripped. This way, multiline strings can
be indented for nicer code formatting without producing unwanted whitespaces in
the output. For example:

```nickel #repl
> m%"
    This line has no indentation.
      This line is indented.
        This line is even more indented.
    This line has no more indentation.
  "%
"This line has no indentation.\n  This line is indented.\n    This line is even more indented.\nThis line has no more indentation."
```

The only special sequence in a multiline string is the string interpolation:

```nickel #repl
> m%"Multiline\nString?"%
"Multiline\\nString?"

> m%"Multiline%{"\n"}String"%
"Multiline\nString"
```

A multiline string can be opened and closed with multiple `%` signs, as long as
the number of `%` signs in the start delimiter equals the number in the closing
delimiter. If you want to use string interpolation, you must use the same amount
of `%` signs as in the delimiters. This can be useful for writing a literal `"%`
or `%{` sequence in a string without escaping:

```nickel #repl
> m%%"Hello World"%%
"Hello World"

> m%%%%%"Hello World"%%%%%
"Hello World"

> let w = "World" in m%%"Hello %{w}"%%
"Hello \%{w}"

> let w = "World" in m%%"Hello %%{w}"%%
"Hello World"
```

Multiline string interpolation is "indentation-aware". This means that you can
interpolate a string with indentation and the result will be as expected:

```nickel #repl
> let log = m%"
  if log:
    print("log:", s)
  "% in m%"
  def concat(str_array, log=false):
    res = []
    for s in str_array:
      %{log}
      res.append(s)
    return res
  "%
"def concat(str_array, log=false):\n  res = []\n  for s in str_array:\n    if log:\n      print(\"log:\", s)\n    res.append(s)\n  return res"
```

Inside a multiline string, an interpolation sequence immediately preceded by a
double quote - that is of the form `"%..%{` - is always interpreted as a
potential string literal prefix `"%..%` followed by an interpolation sequence
opening delimiter, **even if the leading `"%..%` could also act as a string end
delimiter**:

```nickel #repl
> let msg = "Hello, world!" in m%"
    echo "%{msg}"
  "%
"echo \"Hello, world!\""
```

Strictly speaking, the `"%` prefix of `"%{msg}` could be interpreted as the end
of the multiline string. However, doing so makes the common case of interpolating
a variable enclosed in double quotes annoying. Moreover, when such an ambiguity
arises, choosing to eagerly interpret `"%` as a closing delimiter never produces
a meaningful expression. Hence, multiline string ending which is followed by
zero or more `%` and an opening brace `{` is never interpreted as a string
ending.

#### Escape sequences

A backslash `` \ `` followed by one or more characters is interpreted as an
escape sequence. The following escape sequences are supported:

|    Escape sequence | Description                                    |
|:------------------:|:-----------------------------------------------|
| `\\`               | Backslash `` \ ``                              |
| `\"`               | Double quote `"`                               |
| `\n`               | New line                                       |
| `\t`               | Tab                                            |
| `\r`               | Carriage return                                |
| `\%`               | Percent sign `%`                               |
| `\x[a-zA-Z0-9]{2}` | Ascii code sequence (e.g. `\x20` is a space)   |

#### Symbolic Strings

Some tools targeted by Nickel require manipulating string-like values that are
not yet known at the time of evaluation, such as Terraform's computed values.
Others, like Nix, perform additional dependency tracking (see [Nix string
context][nix-string-context]). In both cases, we have to build and combine
string-like values which are more complex than bare strings, but for which using
a string syntax would still feel natural.

That is precisely the use-case for symbolic strings:

```nickel
let inputs = { gcc = "", hello = "", coreutils = "" } in # hide-line
{
  args = [
      "-c",
      nix-s%"
        %{inputs.gcc}/bin/gcc %{inputs.hello} -o hello
        %{inputs.coreutils}/bin/mkdir -p $out/bin
        %{inputs.coreutils}/bin/cp hello $out/bin/hello
      "%,
  ],
  ..
}
```

This example is an excerpt of a Nix configuration written in Nickel, emulating
Nix string contexts. The region delimited by `nix-s%"` and `"%"` on lines 4 to
8 is a symbolic string. The values `inputs.gcc`, `inputs.hello`, etc. aren't
actually strings, but arbitrary records, because they carry additional context.
Yet, they can be interpolated as if they were strings.

The idea behind symbolic strings is to offer a string-like syntax, but without
evaluating the expression as a string. Instead, the expression is returned in a
symbolic form - in practice, an array of fragments, where each fragment is
either a string or an arbitrary value that has been interpolated - and Nickel
lets the specific library (Terraform-Nickel, Nix-Nickel, etc.) handle it.

The prefix of a symbolic string is any valid identifier that doesn't start with
`_`, and ends with the suffix `-s`. Prefixes don't have any meaning for Nickel:
they're just a tag used by libraries consuming symbolic strings to distinguish
between several types of symbolic strings. Prefixes are also a visual marker for
the programmer.

Besides the custom prefix, symbolic strings otherwise follow the same syntactic
rules as multiline strings: the prefix is followed by an arbitrary number of `%`
followed by `"`, and must be closed by `"` followed by the same number of `%`.

The technical details don't matter too much in practice. As a user of a library
which uses symbolic strings, remember that:

- a special string with a prefix ending in  `-s` is a symbolic string. The
  prefix (or prefixes) is defined by the library.
- it's a special syntax without pre-existing meaning for Nickel. The
  specific meaning of each kind of symbolic string, and what it's used for
  exactly, is defined by the library. All in all, symbolic strings simply
  provide libraries with a way to overload string syntax and interpolation for
  extended usages.
- the main operation supported by symbolic strings is interpolation: `%{value}`.
  What interpolation means, and which values can be interpolated in a given
  symbolic string is again defined by each library. Other string functions don't
  work on symbolic strings (e.g. `std.string.length`, `std.string.characters`,
  and so on), because they might not have any valid meaning. Instead, libraries
  should export their own string API, if they support additional operations on
  their symbolic strings.

The following examples show how symbolic strings are desugared:

```nickel #repl
> mytag-s%"I'm %{"symbolic"} with %{"fragments"}"%
{
  fragments = [ "I'm ", "symbolic", " with ", "fragments" ],
  prefix = 'mytag,
  tag = 'SymbolicString,
}

> let terraform_computed_field = {
    tag = 'TfComputed,
    resource = "foo",
    field = "id",
  }

> tf-s%"id: %{terraform_computed_field}, port: %{5}"%
{
  fragments =
    [
        "id: ",
        { field = "id", resource = "foo", tag = 'TfComputed, },
        ", port: ",
        5
      ],
  prefix = 'tf,
  tag = 'SymbolicString,
}
```

## Enums

An enumeration value is composed of a tag and an optional argument serving as a
data payload. An enum without an argument is called an *enum tag*. An enum tag
applied to an argument is called an *enum variant*. An *enum* refers to both
without distinction.

### Enum tags

Enumeration tags are used to express a choice among finitely many alternatives.
They are formed by writing a single quote `'` followed by any valid identifier
or by a quoted string. For example, `std.serialize` takes an export format as a
first argument, which is an enum tag among `'Json`, `'Toml` or `'Yaml`:

```nickel #repl
> std.serialize 'Json {foo = 1}
"{\n  \"foo\": 1\n}"

> std.serialize 'Toml {foo = 1}
"foo = 1\n"
```

An enum tag `'foo` is serialized as the string `"foo"`:

```nickel #repl
> std.serialize 'Json {foo = 'bar}
"{\n  \"foo\": \"bar\"\n}"
```

While it's technically possible to just use strings in place of enum tags, using
an enum tag encodes the intent that only a finite number of alternatives can be
used for the corresponding value.

Additionally, the typechecker is aware of enums and can for example statically
enforce that only valid tags are passed to a function within a typed block. See
[the manual section on typing](./typing.md) for more details.

### Enum variants

An enum variant is an enum tag with associated data. It's useful to represent
more elaborate alternatives and to encode structured data. They are formed by
applying an enum tag to one argument:

```nickel #repl
> 'Foo 5
'Foo 5

> 'Greeting ("Hello," ++ " world!")
'Greeting "Hello, world!"

> 'Operation { op_type = 'select, table = "users", clause = 'Where "id=1" }
'Operation { clause = 'Where "id=1", op_type = 'select, table = "users", }
```

A typical example is the result of a function that may raise a non-fatal error:

```nickel #repl
> let first_elem = fun array =>
   if array == [] then
     'Error "empty array"
   else
     'Ok (std.array.elem 0 array)
  in
  first_elem []
'Error "empty array"
```

Enum variants are structured data. They are expected to be used internally to
make nice and ergonomic library APIs.

Enum variants are the only primitive data structure of Nickel that can't be
serialized. Indeed, there is no obvious canonical way to encode enum variants in
the JSON data model (though many such encodings exist). If you need to serialize
and deserialize enum variants, you'll have to explicitly map them to and from
serializable data structures (such as records).

If you want to write a schema for a configuration where a field accepts
different type of values and such that this configuration can be easily
(de)serialized, you can use the `std.contract.any_of`[^any-of-limitations]
combinator instead:

```nickel
let Schema = {
    size | std.contract.any_of [String, Number],
    ..
}
in

{ size = "1MB" } | Schema
```

**Caution**: Although function application and enum "application" share the
same surface syntax, applying an enum tag to an argument in order to form an
enum variant is different from normal function application. In particular, an
enum variant must be *fully applied at the definition site*, or it will be
parsed as a bare enum tag. For example, `let f = 'Ok in f 5` is not equal to
`'Ok 5` and will actually lead to an error reporting that the enum tag `'Ok`
isn't a function and thus can't be applied. If you need to turn a tag into a
variant-producing function, you need to introduce a parameter such that `'Ok` is
fully applied: `let f = fun x => 'Ok x in f 5` successfully evalutes to `'Ok 5`
as expected.

[^any-of-limitations]: `std.contract.any_of` has limitations for non trivial
    contracts. Be sure to read its documentation in order to fully understand
    them.

## Equality

Operators `==` and `!=` are used to compare values. Two values of different
types are never equal: that is, `==` doesn't perform implicit conversions.

Here are some examples of equality comparisons in Nickel:

```nickel #repl
> 1 == 1
true

> 5 == 5.0
true

> "Hello" == "Hello"
true

> "Hello" != "World"
true

> 5 == "Hello"
false

> true == "true"
false
```

## Composite values

### Arrays

An array is a sequence of values. Arrays are delimited by `[` and `]`, and
elements are separated with `,`.

The following are valid Nickel arrays, for example:

```nickel #lines
[1, 2, 3]
["Hello", "World"]
[1, true, "true"]
[]
```

Arrays can be concatenated with the operator `@`:

```nickel #repl
> [1] @ [2, 3]
[ 1, 2, 3 ]
```

### Records

Records are key-value storage, or in Nickel terms, field-value storage. They
are delimited by `{` and `}`, and elements are separated with `,`. A field
definition is written as `field = value`. The fields are strings, but can be
written without quotes `"` if they are valid identifiers. Values can be of
any type. Elements inside a record are unordered. Two records can be *merged*
together using the operator `&`. The reader can find more information about
merging in the [section on merging](./merging.md).

Here are some valid Nickel records:

```nickel #lines
{}
{a = 3}
{my_id_n5 = "my id number 5", "my id n4" = "my id number 4" }
{"5" = 5, six = 6}
```

Record fields can be accessed using the `.` operator :

```nickel #repl
> { a = 1, b = 5 }.a
1

> { a = 1 }.b
error: missing field `b`
[...]

> { "1" = "one" }."1"
"one"
```

It is possible to write records of records via *piecewise syntax*, where we
separate fields by dots:

```nickel #repl
> { a = { b = 1 } }
{ a = { b = 1, }, }

> { a.b = 1 }
{ a = { b = 1, }, }

> { a.b = 1, a.c = 2, b = 3}
{ a = { b = 1, c = 2, }, b = 3, }
```

When fields are enclosed in double quotes (`"`), you can use string
interpolation to create or access fields:

```nickel #repl
> let k = "a" in { "%{k}" = 1 }
{ a = 1, }

> let k = "a" in { a = 1 }."%{k}"
1
```

#### Include expressions

It's common to want to define a record field from a pre-existing variable with
the same name. Unfortunately, doing it naively won't work because records are
recursive by default:

```nickel #repl
nickel> let
  make_user : String -> {name: String, admin: Bool} = fun name =>
    { name = name, admin = false }
  in
  make_user "Alice"
error: infinite recursion
```

Indeed, `name = name` is a interpreted as a self-recursive definition because
the field `name` being defined shadows the original function parameter. A common
work-around is to pick a different identifier for the outer parameter, for
example `fun _name => { name = _name, admin = false}`. However, this is
unsatisfying, it can quickly become unwieldy and it isn't even always possible.

To solve this problem, Nickel provides another way of declaring a field:
`include` expressions. Using `include some_field` in a record literal defines a
field `some_field` whose value is taken from the symbol with the same name in
the outer environment (which doesn't include the record fields of the current
literal). More specifically, `{include some_field, other_field = 2}` is
equivalent to `let _123 = some_field in {some_field = _123, other_field =
2}`, where `_123` would be a generated, unique identifier.

Back to our original example, we can write instead:

```nickel #repl
nickel> let
  make_user : String -> {name: String, admin: Bool} = fun name =>
    { include name, admin = false }
  in
  make_user "Alice"
{ admin = false, name = "Alice", }
```

You can include multiple fields at once using the list syntax: `include [x, y,
z]`. The elements of the list must be valid identifiers. A single include can
also be given metadata as in `{include x | Number | doc "An x"}`. Metadata
annotations on include lists is currently not supported.

## Constructs

### If-Then-Else

This construct allows conditional branching in your code. You can use it like
`if <bool expr> then <expr> else <expr>`.

Here are some valid conditional expressions in Nickel:

```nickel #repl
> if true then "TRUE :)" else "false :("
"TRUE :)"

> if false then "Not this one" else "This one"
"This one"

> if "forty-two" == 42 then "equal?" else "unequal"
"unequal"

> ["1"] @ (if 42 == "42" then ["3"] else ["2"]) @ ["3"]
[ "1", "2", "3" ]
```

### Let-In

A `let` binds expressions to variables. It is used like `let <rec?> <ident> =
<expr> in <expr>`, or with multiple comma-separated bindings like
`let <rec?> <ident> = <expr>, <ident> = <expr> in <expr>`. The `rec` keyword
makes the binding recursive, enabling the use of `<ident>` within the bound
expression(s) `<expr>`.

Here are some examples of let bindings in Nickel:

```nickel #repl
> let r = { a = "a", b = "b" } in r.a
"a"

> let inner = { inside = true } in let outer = { outside = inner.inside } in outer.outside
true

> let rec inner = { inside = true }, outer = { outside = inner.inside } in outer.outside
true

> let a = 1, b = 2 in a + b
3

> let rec f = fun n => if n == 0 then n else n + f (n - 1) in f 10
55

> let rec fib = fun n => if n <= 2 then 1 else fib (n - 1) + fib (n - 2) in fib 9
34

> let rec repeat = fun n x => if n <= 0 then [] else repeat (n - 1) x @ [x] in
    repeat 3 "foo"
[ "foo", "foo", "foo" ]
```

## Functions

A function is declared using the `fun` keyword, then arguments separated with
spaces, and finally an arrow `=>` to add the body of the function. To call a
function, just write the arguments after it separated with spaces. Functions in
Nickel are curried: a function taking multiple arguments is actually a function
that takes a single argument and returns a function taking the rest of the
arguments, and so on.

Here are some examples of function definitions in Nickel:

```nickel #repl
> (fun a b => a + b) 1 2
3

> let add = fun a b => a + b in add 1 2
3

> let add = fun a b => a + b in
  let add1 = add 1 in
  add1 2
3
```

All existing infix operators in Nickel can be turned into functions by putting
them inside parentheses, for example:

```nickel #repl
> 1 + 2
3

> (+) 1 2
3

> let increment = fun n => (+) 1 n in
  increment 41
42

> let increment = (+) 1 in
  increment 41
42

> let flatten = std.array.fold_right (@) [] in
  flatten [[1, 2], [3], [4, 5]]
[ 1, 2, 3, 4, 5 ]
```

Functions may be composed using the *pipe operator*. The pipe operator allows
for a function application `f x` to be written as `x |> f`. This operator is
left-associative, so `x |> f |> g` will be interpreted as `g (f x)`. For example:

```nickel #repl
> "Hello World" |> std.string.split " "
[ "Hello", "World" ]

> "Hello World"
  |> std.string.split " "
  |> std.array.first
"Hello"

> "Hello World"
  |> std.string.split " "
  |> std.array.first
  |> std.string.uppercase
"HELLO"
```

## Primitive operators

Primitive operators are low-level native functions that are built into the
interpreter. Primitive operators are enclosed between two `%` signs, as in
`%array/length%`. They have a fixed number of arguments and must always be fully
applied. For example, `%array/length%` takes one argument, and writing `let op =
%array/length% in op []` is a syntax error, while `let length = %array/length%
[] in length` is valid. As some primitive operators return functions, they can
sometimes be applied to more arguments than their official arity, but never
less.

Primitive operators are strict in their arguments, meaning that they force the
evaluation of their arguments before performing any further action. Primitive
operators are internal and you should never use them directly unless you really
know what you are doing. They aren't part of any stability guarantee, as they
can be added, removed or modified at any minor version update. All useful
primitive operators are wrapped as a stdlib function, which has documentation,
stability guarantees, can be partially applied, etc.

They are mentioned here so that the reader isn't lost if they ever encounter
them, e.g. reading the implementation of the standard library or when they
appear in error messages.

## Pattern matching

Match expressions and destructuring offer an ergonomic way to check and to
decompose structured data. Both match expressions and destructuring share the
same language of patterns, described in the following section.

### Patterns

A pattern starts with an optional alias of the form `<ident> @ <inner
pattern>`. The inner pattern is either:

- an `any` pattern, which is just an identifier that will match any value.
    `any` patterns bring a new variable into scope and can be nested inside a
    larger pattern. Said variables are bound to the corresponding constituent
    parts of the matched value.
- a wildcard pattern `_` which acts like an `any` pattern (matches
  anything) but doesn't bind any variable.
- a constant pattern, which is a literal value: a number, a boolean, a string,
  or `null`.
- an enum pattern
- a record patern
- an array pattern
- an or-pattern

Additionally, patterns can be guarded by an `if` condition. Enum patterns, record
patterns, array patterns, or-patterns and pattern guards are detailed in the
following subsections. Complete examples of patterns are given in the section on
match expressions and destructuring.

#### Enum pattern

An enum pattern is an enum tag optionally applied to a pattern: `'<tag> <pat?>`.
An enum pattern looks exactly like an enum value but the optional argument is
another pattern instead of an arbitrary expression. This pattern matches an enum
value of the corresponding shape.

For example, `'Foo`, `'Bar x` or `'protocol {x,y}` are valid enum patterns. For
example, they will match values `'Foo`, `'Bar 5` and `'protocol {x = 1, y = 2}`
respectively.

Two or more nested variant patterns must be parenthesized. For example, `'Ok
'Some 'Stuff` isn't a valid enum pattern. On the other hand, `'Ok ('Some
'Stuff)` and `'Foo ('Bar x)` are valid enum patterns.

#### Record patterns

Similarly to other compound patterns, record patterns mimic the shape of record
literals. A record pattern is a list of field patterns enclosed into braces of
the form `{ <field_pat1>, .., <field_patn>, <rest?> }`.

A field pattern is of the form `<ident> <annot?> = <pat>`, where `<pat>` is a
sub-pattern matching the content of the field. For example, `foo = bar` and
`foo = 'Ok value` are valid field patterns. The `= <pat>` part can be omitted when
`<pat>` is an `any` pattern with the same name as the field: that is,
`some_field` is a valid field pattern and is just shorthand for
`some_field = some_field`.

The optional annotation `<annot>` can include either:

- A contract annotation. For example, `foo | Number = bar` or
  `source | [| 'Link Url |] = 'Link url`.
- A default value, introduced with `?`. For example, `bar ? 5` or
  `baz ? {} = {qux ? false, corge ? null}`.

A contract annotation and a default annotation can be combined.

**The presence or the absence of a contract annotation never changes whether or
not a pattern matches a value**. For example, all of `{foo}`, `{foo | Number}`
and `{foo | String}` match the value `{foo = "hello"}`. The difference is that
`{foo | Number}` will result in a later contract error if `foo` is ever used.
The contract annotation is merely a convenient way to apply a contract to a
value extracted from the pattern match on the fly.

On the other hand, a default annotation does make a difference on matching:
`{foo ? 5}` matches `{}` (and will bind `foo` to the default value `5`), but the
pattern `{foo}` doesn't match `{}`. Note that default values don't propagate to
aliases: `whole @ {foo ? 5}` will match `{}` and assigns `whole` to `{}` and
`foo` to `5`. Note that `whole` is *not* `{foo = 5}`.

The optional `<rest?>` part is either an ellipsis `..` or a capture `..<ident>`.
By default, record patterns are closed, meaning that they won't match a record
with additional fields: `{foo, bar}` doesn't match `{foo = 1, bar = 2, baz =
3}`.

The ellipsis `..` makes the pattern open. An open pattern matches a record with
additional fields. A capture has the same effect but also captures the rest of
the matched record in a variable. For example, matching `{foo, ..rest}` with
`{foo = 1, bar = 2, baz = 3}` will bind `foo` to `1` and `rest` to the record
`{bar = 2, baz = 3}`.

#### Array pattern

An array pattern is a list of patterns enclosed into brackets, of the form `[
<pat1>, .., <patn>, <rest?> ]`. The rest is either an ellipsis `..` or a capture
`..<ident>`, as for record patterns. An array pattern without a rest will match
an array value with exactly the same number of elements, and whose elements
individually match the corresponding patterns. The presence of the rest allows
for more elements to be there.

For example, `[first, ..rest]`, `[true, _, true]` and `[x, y, z, ..]` are valid
array patterns.

#### Or-patterns

An or-pattern is a pattern with several alternatives. An or-pattern is
introduced by the `or` keyword, as in `<pattern1> or <pattern2>`. Note that `or`
isn't a reserved keyword in Nickel (`or` can be used as an identifier, including
as a pattern variable, without any ambiguity).

Each alternative is called an or-pattern branch. All the branches of an
or-pattern must bind exactly the same variables: that is, `('Foo x) or ('Bar y)`
isn't a valid or-pattern, but `'(Foo {x,y}) or ('Bar {y,x})` is.

Aliases aren't allowed at the top-level of an or-pattern branch. For example,
`(x @ {}) or {x}` isn't a valid or-pattern. You can still alias the whole
or-pattern: `x @ (('Foo y) or ('Bar y))` is a valid or-pattern.

Additionally, enum variant patterns must be parenthesized at the top-level of an
or-pattern branch for readability reasons. For example, `'Foo x or 'Bar x` isn't
a valid or-pattern, but `('Foo x) or ('Bar x)` is. Similarly, `'Par or or 'Plus
or` isn't a valid or-pattern, but `('Par or) or ('Plus or)` is (in this case,
the `or` inside the parentheses is just a normal pattern variable).

Or-patterns can optionally be parenthesized when needed, as in `({..} or [..])`.

#### Pattern guards

A pattern guard is an optional boolean condition which is attached to a pattern
in a match expression. Note that pattern guards aren't allowed for destructuring
and they can't appear nested in a larger pattern. A guard is introduced by the
`if` keyword, as in `<pattern> if <condition>`. The condition is a Nickel
expression which can use the variables bound by the pattern and must evaluate to
a boolean.

For example, `{tag = _, value = 'Wrapped x} if std.is_number x && x > 0` is a
valid guarded pattern. This pattern will match `{tag = 'Cut, value = 'Wrapped
5}` but not `{tag = 'Cut, value = 'Wrapped (-5)}`.

### Match expressions

A match expression is a control flow construct which checks a value against one
or more patterns. The first successful match binds the pattern variables to the
corresponding constituent parts. When applicable, match expressions can
succinctly and advantageously replace long or complex sequences of if-then-else.

A match expression behaves as a function. It must be applied to the value to
check. A match expression is introduced by the `match` keyword, followed by a
sequence of match arms enclosed by braces. Patterns can be guarded by an
additional condition.

```text
match {
  <pat1> <if expr>? => <exp1>,
  ...,
  <patn> <if expr>? => <expn>,
}
```

Examples:

```nickel #repl
> 5 |> match {x => x + 1}
6

> {x = 1, y = 2} |> match {
    {x,z} => null,
    {x,y} => x + y,
    {y,z} => null
  }
3

> let display = match {
    'Ok msg => "It's ok: %{msg}!",
    'Error err => "It's not ok :( (%{err})",
    _ => "Unexpected value"
  }
  in
  [ display ('Ok "good"), display ('Error "bad"), display 'Other ]
[ "It's ok: good!", "It's not ok :( (bad)", "Unexpected value" ]

> {type = 'binary, format = 'elf32, meta.editor = "SuperCompany"} |> match {
    {format = 'elf64, ..} => 'Error "Unsupported 64 bits format",
    {format = 'elf32, ..rest} => 'Ok rest,
  }
'Ok { meta = { editor = "SuperCompany", }, type = 'binary, }

> [1, 2, 3, 4] |> match {
    [x] => 'Singleton x,
    [x, y] => 'Pair {fst = x, snd = y},
    [x, y, ..rest] => 'PairAndTail {fst = x, snd = y, tail = rest},
  }
'PairAndTail { fst = 1, snd = 2, tail = [ 3, 4 ], }

> {pin = "1234", security = 'Ecc} |> match {
    {pin, ..}
      if std.is_string pin && std.string.is_match "^\\d{4}$" pin =>
      'Ok,
    {pin, ..} if std.is_string pin => 'Error "Pin must be 4 digits",
    {pin, ..} => 'Error "Pin must be a string",
  }
'Ok
```

### Destructuring

Destructuring is an extension of the basic let-binding mechanism to deconstruct
a structured value.

Destructuring can take place on a let binding with the form `let <pat> = value
in <exp>` or at a function declaration with the form `fun <pat1> .. <patn> =>
<exp>`.

Destructuring is just a shorthand for a match expression with a single arm. That
is, `let <pat> = value in <exp>` is equivalent to `value |> match { <pat> =>
<exp> }`. `fun <pat> => <exp>` is equivalent to `fun x => let <pat> = x in
<exp>`. If the pattern doesn't match the value, an unmatched pattern error is
raised.

Destructuring function arguments requires additional parentheses for enum
patterns and or-patterns. Indeed, `fun 'Foo x => <body>` might be ambiguous: it
can be either a function of one argument expecting a value of the form `'Foo x`,
that is an enum variant with an enum tag as an argument, or a function of two
arguments expecting the first one to be the enum tag `'Foo`. To avoid the
confusion, enum variant patterns and or-patterns must be parenthesized in
argument position. That is, `fun 'Foo x` is thus a function of two arguments and
`fun ('Foo x) => <body>` is a function of one argument.

Examples:

```nickel #repl
> let {x, y, z} = {x = 1, y = 1, z = 1} in x + y + z
3

> let top @ {value} = {value = 1} in top & {duplicate = value}
{ duplicate = 1, value = 1, }

> let 'Some {left, right = {..}} = 'Some {left = "left", right = {value = "right"}} in left
"left"

> let f = fun {deps ? [], parent ? null, children ? []}  => deps @ children
  in
  f {deps = ["binutils"]}
[ "binutils" ]

> let f = fun {wrapped=w1} {wrapped=w2} {wrapped=w3} => w1 + w2 + w3
  in
  f {wrapped=1} {wrapped=10} {wrapped=100}
111

> let {x | std.enum.TagOrString} = {x = "Hello"} in x
'Hello

> let [head, ..tail] = [1, 2, 3] in tail
[ 2, 3 ]

> let 'Invalid x = {} in x
error: destructuring failed
[...]
```

## Annotations

Contract and type annotations help enforce additional properties of an
expression. They can be attached to any Nickel expression. See [the correctness
section](./correctness.md) for more details.

### Type annotations

A type annotation is introduced using `<expr> : <type>` and serves to delimit
a statically typed block which will be checked by the typechecker before
evaluation. A type annotation can be directly attached to the variable of a let-
binding `let <var> : <type> = <expr> in <body>` or to a record field declaration
`{<field> : <type> = <value>}` as well.

A type wildcard `_` indicates that part of a type is unknown to the user (or
is not worth spelling out). The typechecker will attempt to infer it. Adding a
wildcard type annotation `: _` to an existing expression is particularly useful
for debugging, as it's the simplest way to have the typechecker run on an
expression without having to come up with a type.

Here are some examples of type annotations in Nickel:

```nickel #repl
> 5 : Number
5

> "hello" : String
"hello"

> "Hello," ++ " world!" : String
"Hello, world!"

> 5 + "a" : _
error: incompatible types
[...]

> let result : Number = 1 + 1 + ('foo |> match { 'foo => 1, _ => 2 }) in
  result
3

> let x : Number = "a" in x
error: incompatible types
[...]

> let complex_ar : _ -> Number = fun {field1, field2, field3} => field1 in
  complex_ar {field1 = 5, field2 = null, field3 = false}
5
```

### Contract annotations

A contract annotation is introduced by `<exp> | <contract>` and serves to apply
a runtime check to an expression (among other things).

As detailed in the next section, `<expr>`, `<type>` and `<contract>` are in fact
syntactically all the same and can be arbitrary Nickel expressions in practice.

Here are some examples of contract annotations in Nickel:

```nickel #repl
> 5 | Number
5

> 5 | Bool
error: contract broken by a value
[...]

> let SmallNumber = std.contract.from_predicate (fun x => x < 5) in
  1 | SmallNumber
1

> let SmallNumber = std.contract.from_predicate (fun x => x < 5) in
  10 | SmallNumber
error: contract broken by a value
[...]

> let SmallNumber = std.contract.from_predicate (fun x => x < 5) in
  let NotTooSmallNumber = std.contract.from_predicate (fun x => x >= 2) in
  3 | Number
    | SmallNumber
    | NotTooSmallNumber
3
```

### Types

The Nickel syntax mixes both terms and types in the same namespace. The
following program is perfectly legal: `let value = Number -> (fun value label =>
value) in ((fun x => x + 1) : value)`. See the
[RFC002](https://github.com/tweag/nickel/blob/d723a3721c6b0fe9c4b856e889bb7211d6136665/rfcs/002-merge-types-terms-syntax.md)
for a detailed account of this design.

The documentation still makes a distinction between *types* and other
expressions, the former being constructs which are handled specially by the
typechecker and are listed below. However, any expression can be considered a
type (in the generic case, it will be considered as an opaque type), and type
constructors can also appear inside an expression (where they are understood as
their associated contract, which is indeed an expression, most often a
function).

Thus, placeholders such as `<source>`, `<target>` or `<type>` can actually be
substituted with any valid Nickel expression (which includes the type
constructors we've just listed), and types can appear anywhere.

Nickel features the following builtin types and type constructors:

- Primitive types: `Number`, `String`, `Bool`, and `Dyn` (the dynamic type, which
represents any value)
- Arrays: `Array <type>` is an array whose elements are of type `<type>`.
- Dictionaries: `{_ : <type>}` is a record whose fields are of type `<type>`.
- Enums: `[| 'tag1 <type1?>, .., 'tagn <typen?>|]` is an enumeration comprised of
  alternatives. Constituents have the same syntax as enum values: they can be
  either unapplied (like enum tags) or applied to a type argument (like enum
  variants). They are prefixed with a single quote `'`. Like record fields, they
  can also be enclosed in double quotes if they contain special characters:
  `'"tag with space"`.
- Arrows: `<source> -> <target>` is a function taking an argument of type
  `<source>` and returns values of type `<target>`.
- Foralls: `forall var1 .. varn. <type>` is a polymorphic type quantifying over type
variables `var1`, .., `varn`.
- Records: see the next section [Record types](#record-types).

Type variables bound by a `forall` are only visible inside types (any of the
constructor listed above). As soon as a term expression appears under a `forall`
binder, the type variables aren't in scope anymore:

```nickel #repl
# skip output check hide-line
> forall a. a -> (a -> a) -> {_ : {foo : a}}
<func>

> forall a. a -> (a -> (fun x => a))
error: unbound identifier `a`
[...]
```

Here are some examples of more complicated types in Nickel:

```nickel #repl
> let f : forall a. a -> a = fun x => x in (f 5 : Number)
5

> {foo = [[1]]} : {foo : Array (Array Number)}
{ foo = [ [ 1 ] ], }

> let select
    : forall a. {left: a, right: a} -> [| 'left, 'right |] -> a
    = fun {left, right} =>
       match {
         'left => left,
         'right => right,
       }
  in
  (select {left = true, right = false} 'left) : Bool
true

> let map_ok
  : forall a b err. [| 'Ok a, 'Err err |] -> (a -> b) -> [| 'Ok b, 'Err err |]
  = fun result f =>
    result |> match {
      'Ok value => 'Ok (f value),
      'Err e => 'Err e,
    }
  in
  map_ok ('Ok 1) ((+) 1) : forall e. [| 'Ok Number, 'Err e |]
'Ok 2

> let add_foo : forall a. {_: a} -> a -> {_: a} = fun dict value =>
    std.record.insert "foo" value dict
  in
  add_foo {bar = 1} 5 : _
{ bar = 1, foo = 5, }

> {foo = 1, bar = "string"} : {_ : Number}
error: incompatible types
  ┌─ <repl-input-96>:1:18
  │
1 │  {foo = 1, bar = "string"} : {_ : Number}
  │                  ^^^^^^^^ this expression
  │
  = Expected an expression of type `Number`
  = Found an expression of type `String`
  = These types are not compatible
```

#### Record types

Record types are syntactically a restricted subset of record literals. They are
handled differently than normal record literals with respect to typechecking.

A record literal is a record type if:

- No field has a defined value: there are only fields without definition.
- Each field has exactly one type annotation
- Each field doesn't have any other metadata attached (see [Metadata](#metadata))

If these properties are satisfied, a record literal is considered to be a
record type by the typechecker.

A record literal which is interpreted as a record type may have a *record
tail*. A tail is written like `{ <fields> ; <tail> }`. It appears at the end of
the field declarations and is preceded by `;`. The tail `<tail>` itself must be
a valid identifier.

Trying to attach a tail `; tail` to a record literal which isn't a record type
is a parse error.

Here are some examples of record types in Nickel:

```nickel #repl
> {foo = 1, bar = "foo" } : {foo : Number, bar: String}
{ bar = "foo", foo = 1, }

> {foo.bar = 1, baz = 2} : {foo: {bar : Number}, baz : Number}
{ baz = 2, foo = { bar = 1, }, }
```

Here, the right-hand side is missing a type annotation for `baz`, so it doesn't
qualify as a record type and is parsed as a record contract. This throws an
"incompatible types" error:

```nickel #repl
> {foo = 1, bar = "foo" } : {foo : Number, bar : String, baz : Bool}
error: type error: missing row `baz`
[...]
```

If there's a metadata annotation apart from the type, the record cannot be
parsed as a type. Consequently, Nickel tries to interpret it as a record
contract which will most likely result in an error, because fields with a type
annotation but no value are forbidden outside of types.

```nickel #repl
> {foo = 1, bar = "foo" } : {foo : Number, bar : String | optional}
error: statically typed field without a definition
  ┌─ <repl-input-100>:1:29
  │
1 │  {foo = 1, bar = "foo" } : {foo : Number, bar : String | optional}
  │                             ^^^   ------ but it has a type annotation
  │                             │
  │                             this field doesn't have a definition
  │
  = A static type annotation must be attached to an expression but this field doesn't have a definition.
  = Did you mean to use `|` instead of `:`, for example when defining a record contract?
  = Typed fields without definitions are only allowed inside record types, but the enclosing record literal doesn't qualify as a record type. Please refer to the manual for the defining conditions of a record type.
```

While in the following `MyDyn` isn't a proper type, the record literal `{foo :
Number, bar : MyDyn}` respects all the requirements for a record type and is
parsed as such:

```nickel #repl
> let MyDyn = fun label value => value in
    {foo = 1, bar | MyDyn = "foo"} : {foo : Number, bar : MyDyn}
{ bar | MyDyn = "foo", foo = 1, }
```

## Metadata

Metadata annotations are used to attach type and contract annotations,
documentation, a merge priority or other decorations to record fields (and
record fields only). Multiple metadata annotations can be chained. Metadata
is introduced with the syntax `<field_name> | <metadata1> | .. | <metadataN>
[= value]`.

Documentation can be attached with `| doc <string>`. For example:

```nickel #repl
> let record = {
    value
      | doc "The number five"
      | default
      = 5
  }

# Stop `core/tests/manual` from parsing this hide-line
> :query record value
• default: 5
• documentation: The number five

> {
    truth
      | Bool
      | doc m%"
          If something is true,
          it is based on facts rather than being invented or imagined,
          and is accurate and reliable.
          (Collins dictionary)
        "%
      = true,
  }.truth
true
```

Metadata can also set merge priorities using the following annotations:

- `default` is the lowest priority, usually used for default values that are
  expected to be overridden somewhere
- `priority NN`, where `NN` is a number literal, is a numeral priority
- `force` is the highest priority

If there is no priority specified, `priority 0` is the default. See more
about this in the [dedicated section on merging](./merging.md).

Here are some examples using merge priorities in Nickel:

```nickel #repl
> let Ais2ByDefault = { a | default = 2 } in
    {} | Ais2ByDefault
{ a | default = 2, }

> let Ais2ByDefault = { a | default = 2 } in
    { a = 1 } | Ais2ByDefault
{ a = 1, }

> { foo | default = 1, bar = foo + 1 }
{ bar = 2, foo | default = 1, }

> {foo | default = 1, bar = foo + 1} & {foo = 2}
{ bar = 3, foo = 2, }

> {foo | force = 1, bar = foo + 1} & {foo = 2}
{ bar = 2, foo | force = 1, }

> {foo | priority 10 = 1} & {foo | priority 8 = 2} & {foo = 3}
{ foo | priority 10 = 1, }

> {foo | priority -1 = 1} & {foo = 2}
{ foo = 2, }
```

The `optional` annotation indicates that a field is not mandatory. It is usually
found in record contracts.

```nickel #repl
> let Contract = {
    foo | Number,
    bar | Number
        | optional,
  }

> let value | Contract = {foo = 1}

> value
{ foo | Number = 1, }

> {bar = 1} | Contract
error: missing definition for `foo`
[...]
```

The `not_exported` annotation indicates that a field should be skipped when a
record is serialized. This includes the output of the `nickel export` command:

```nickel #repl
> let value = { foo = 1, bar | not_exported = 2}

> value
{ bar = 2, foo = 1, }

> std.serialize 'Json value
"{\n  \"foo\": 1\n}"
```

## Imports

A Nickel program can import other Nickel files using the `import` keyword: `let
lib = import "lib.ncl" in lib.base64_encode [01, 02, 03]`. Nickel can import
other Nickel files, but also JSON, TOML, YAML, or raw text.

There is special keyword `import`, which can be followed by either a string
literal or an enum tag and a string literal.

This causes Nickel to read, evaluate and return the specified file.

The file is searched in directories specified by `NICKEL_IMPORT_PATH`
environment variable or similar command line option, with default being the
current directory.

One-argument import, like `import "myfile.ncl"`, uses filename extension to
determine the file format. Nickel automatically recognizes the extensions
`ncl`, `json`, `yml`, `yaml`, `toml` and `txt`. When compiled with experimental Nix
support, it also recognizes `nix`. If the file's extension is not recognized, it
will default to Nickel format.

Two-argument import, like `import "test.html" as 'Text` uses a special enum
tag to determine the format. Currently the tags are `'Nickel`, `'Json`,
`'Yaml`, `'Toml`, `'Text` and `'Nix`. Some of the formats may be unavailable
depending on compilation options of the Nickel interpreter.

[nix-string-context]: https://shealevy.com/blog/2018/08/05/understanding-nixs-string-context/
