# Mova

An interpreted language designed around ownership and safe memory management.

## Examples

Here are some examples to get you started:

### Variables

```
// We use 'let' keyword to declare a variable.
// By default, variables in Mova are immutable (read-only).
let x = 123
let y = 111
```

### Functions
```
// We use 'fn' keyword to declare a function.
// Mova is an expression-based language. The result of the last expression
// is returned automatically.
fn add(a, b) = a + b
let result = add(x, y)
```

### Scope and Shadowing
```
// We can declare 'x' again using the same 'let' keyword.
// This "shadows" (hides) the previous definition of 'x'.
// This is useful for transforming data without creating new variable names.
let x = 666

// Code blocks declared with '{' and '}' create a new scope.
// Variables declared inside are isolated from the outside.
let scoped_value = {
    let inner = 10
    // The block evaluates to this expression
    x + inner
}
// 'inner' is no longer accessible here, but 'scoped_value' is 676
```

### References and Borrowing
```
// '&' creates an immutable reference (a borrow).
// This allows you to read data without taking ownership of it.
fn echo(value) = value

let data = 100
let reference = &data

// We use '*' to explicitly dereference.
let calculation = *reference * 2
```

### Mutation
```
// To make a variable modifiable, use the 'mut' keyword.
let mut counter = 0
counter = 1

// You can also create mutable references using '&mut'.
// This allows a function to modify a value owned by someone else.
fn increment(value) = {
  value = value + 1
}

// When passing mutable references, the borrow checker ensures safety
increment(&mut counter)
```

## License

Mova is distributed under the terms of [MIT License](./LICENSE).
