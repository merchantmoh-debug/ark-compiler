# The Ark-1 Programmer's Field Manual

**Welcome to the Ark.**
Ark-1 is a sovereign, distinct programming language designed for **verifiable computing**. Unlike Python or JavaScript, every piece of Ark code is hashed (Content-Addressable) and executed in a totally isolated WebAssembly runtime.

This manual will teach you how to think in Ark.

## 1. The Philosophy: Code is Logic
Ark is a **Lisp-like** language. This means:
1.  **Everything is a List**: `(function arg1 arg2)`.
2.  **Prefix Notation**: Operations come first. `1 + 2` is `(+ 1 2)`.
3.  **Strictly Typed (Under the Hood)**: While the syntax feels dynamic, the runtime enforces strict ownership logic (Linear Types).

> [!TIP]
> **Try It Live**: All examples below can be typed directly into the [Live Demo](https://merchantmoh-debug.github.io/ark-compiler/).

---

## 2. Basic Forms

### Printing
The classic Hello World.
```scheme
(print "Hello Ark")
```

### Arithmetic
Math uses prefix notation. It removes ambiguity (no order of operations confusion).
```scheme
(print (+ 10 20))       ;; Prints 30
(print (* 5 (+ 2 3)))   ;; Prints 25 -> 5 * (2 + 3)
```

---

## 3. Variables (`let`)
Variables in Ark are immutable by default in some contexts, but `let` binds a value to a name in the current scope.

```scheme
(let x 10)
(print x)

(let y (* x 2))
(print y)
```

---

## 4. Logic and Control Flow (`if`)
Ark uses conditional expressions. An `if` statement evaluates a condition. If true, it executes the first block. If false, the second.

### Syntax
`(if condition then_block else_block)`

### Examples
```scheme
(if (< 10 20)
    (print "Math is Real")
    (print "Logic is Broken")
)
```

---

## 5. Loops (`while`)
The `while` loop repeats a body of code as long as a condition is true.

> [!NOTE]
> In the current web demo version, be careful with infinite loops!

```scheme
(let i 0)
(while (< i 5)
    (print i)
    (let i (+ i 1))
)
```

---

## 6. Functions (`fn`)
Defining your own logic is the core of sovereign computing. In Ark, you define functions with `fn`.

### Syntax
`(fn name (arg1 arg2) body...)`

### Example: The Adder
```scheme
(fn add (a b)
    (+ a b)
)

(print (add 10 20))
```

### Example: Recursive Factorial
Ark supports recursion (functions calling themselves).

```scheme
(fn factorial (n)
    (if (< n 2)
        1
        (* n (factorial (- n 1)))
    )
)

(print (factorial 5)) ;; Prints 120
```

---

## 7. The Sovereign Future
You have learned the basics of Ark-1.
- You understand **S-Expressions**.
- You can control flow with **If** and **While**.
- You can encapsulate logic with **Fn**.

This is just the beginning. The Ark compiler is designed to evolve into a self-hosting system where the code you write is mathematically proven to be correct.

**End of Manual.**
