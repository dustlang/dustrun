# Expression Semantics (v0.1)

Source: `crates/dvm/src/lib.rs` (`expr` module)

## Lexer Tokens

- identifiers
- ints
- bools (`true`, `false`)
- strings with escapes (`\"`, `\\`, `\n`, `\t`, `\r`)
- punctuation: `(` `)` `,` `.` `{` `}` `:`

## Value Evaluation

Evaluator returns DVM `Value`:

- `Int`
- `Bool`
- `String`
- identifier lookups from env

## Operator Identifiers

Binary operators are words, not symbols:

- arithmetic: `Add`, `Sub`, `Mul`, `Div`
- compare: `Eq`, `Lt`, `Le`, `Gt`, `Ge`
- boolean: `And`, `Or`

## Precedence

Highest to lowest:

1. primary
2. `Mul` / `Div`
3. `Add` / `Sub`
4. comparisons
5. `And`
6. `Or`

## Error Behavior

Examples of runtime evaluator errors:

- unknown identifier,
- invalid int literal,
- division by zero,
- type mismatch for operators,
- unexpected tokens.
