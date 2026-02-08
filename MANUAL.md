# The Ark-1 Programmer's Field Manual

**Version:** 1.0 (Sovereign Edition)
**Author:** Mohamad Al-Zawahreh
**License:** AGPLv3

---

## Preface: The Sovereign Philosophy

Welcome to the Ark.

You are about to learn **Ark-1**, a language designed not for convenience, but for **Sovereignty**.
In a world of black-box AI and rental computing, Ark is a return to first principles.

### The Three Laws of Ark
1.  **Code is Data:** Every program is a Merkle Tree (MAST). Code is content-addressable and immutable.
2.  **State is Explicit:** There is no hidden garbage collection magic. You own your memory.
3.  **Execution is Verifiable:** The runtime is deterministic. If it runs on your machine, it runs on Mars.

This manual will take you from a blank slate to building self-replicating logic.

---

## Chapter 1: The Environment

Before we write code, we must know where to run it.
Ark runs in a secure **WebAssembly Runtime**.

### The Sovereign Site
The easiest way to learn is the [Certified Browser Terminal](https://merchantmoh-debug.github.io/ark-compiler/).
- **The Input:** A standard command line.
- **The Engine:** A pure WASM blob executing your logic client-side. Nothing is sent to a server (unless you ask it to).

**Try it now:**
Open the terminal and verify the system is live.
```scheme
(print "System Online")
```

---

## Chapter 2: Syntax & Structure

Ark is a **Lisp-dialect**. If you have used Scheme or Clojure, this will feel familiar. If you come from Python or C++, shift your mindset.

### S-Expressions
Everything in Ark is a list. A list is wrapped in parentheses `(...)`.
The first item in the list is the **Function**, and the rest are **Arguments**.

**Python/C Style:**
`function(arg1, arg2)`

**Ark Style:**
`(function arg1 arg2)`

### Prefix Notation
Math follows the same rule. The operator comes first.

```scheme
(+ 10 20)      ;; 10 + 20
(* 5 5)        ;; 5 * 5
(- 100 1)      ;; 100 - 1
```

### Nesting
You can nest lists as deep as you like.
```scheme
(* 10 (+ 5 5)) ;; 10 * (5 + 5) -> 100
```

---

## Chapter 3: primitive Types

Ark-1 supports a strict set of primitives.

### Integers
64-bit signed integers.
`1`, `42`, `-99`

### Strings
UTF-8 text strings, enclosed in double quotes.
`"Hello"`
`"Sovereign integrity"`

### Booleans
Truth values.
`true`
`false`

**Type Checking:**
The runtime enforces types strictly. You cannot add a String to an Integer.
```scheme
(+ "Hello" 10) ;; ERROR: Type Mismatch
```

---

## Chapter 4: Variables & Scope

Data is stored in **Variables**.
Use the `let` keyword to bind a value to a name.

```scheme
(let x 100)
(print x)
```

### Re-binding
You can re-use names in *new* scopes or sequential statements, effectively "updating" the variable from the programmer's perspective.

```scheme
(let x 1)
(let x (+ x 1)) ;; x is now 2
(print x)
```

### Scope
Variables defined inside a block (like an `if` or `fn`) do not exist outside it.

---

## Chapter 5: Control Flow

Logic requires decision making.

### If-Expression
The `if` form evaluates a condition. If `true`, it runs the first block. If `false`, the second.

**Syntax:** `(if condition then-expression else-expression)`

```scheme
(let power 9000)

(if (> power 8000)
    (print "It's over 8000!")
    (print "Needs more training")
)
```

### While-Loop
To repeat an action, use `while`. This requires a concept of "mutable" state or re-binding in the current simple shell.

**Syntax:** `(while condition body...)`

```scheme
(let i 0)
(while (< i 5)
    (print i)
    (let i (+ i 1))
)
```
*> Warning: In the browser, an infinite loop will freeze your tab. Ensure your condition eventually becomes false.*

---

## Chapter 6: Functions

The heart of Ark is the Function. You define new behavior using `fn`.

**Syntax:** `(fn name (args...) body...)`

### Defining a Function
Let's create a function that squares a number.

```scheme
(fn square (x)
    (* x x)
)

(print (square 5)) ;; Prints 25
```

### Multi-Argument Functions
```scheme
(fn area (width height)
    (* width height)
)

(print (area 10 5)) ;; Prints 50
```

### The Implicit Return
The **last expression** in a function body is automatically returned. You do not need a strict `return` keyword (though `return` exists for early exit).

---

## Chapter 7: The Standard Library

Ark comes with a compact standard library.

### Math
- `+` : Addition
- `-` : Subtraction
- `*` : Multiplication
- `/` : Division (Integer)

### Comparison
- `<` : Less Than
- `>` : Greater Than
- `=` : Equal To

### I/O
- `print` : Outputs text to the console.

### The Neuro-Link (v1.0)
Ark is an **AI-Native** language. It has a built-in intrinsic to communicate with the Sovereign Mind (LLM).

```scheme
(ask_ai "What is the capital of Mars?")
```
*> Note: This requires the local bridge to be active. In the browser demo, this serves as a placeholder for future WebLLM integration.*

---

## Chapter 8: Advanced Concepts

### Recursion
Ark functions can call themselves. This is the primary way to express complex iteration without loops.

**Factorial Example:**
```scheme
(fn factorial (n)
    (if (< n 2)
        1
        (* n (factorial (- n 1)))
    )
)

(print (factorial 5)) ;; 120
```

### The Stack
Ark uses a traditional call stack. Each function call pushes a new frame. Deep recursion may overflow the stack if not careful (Maximum depth is currently runtime-dependent).

### Content-Addressability (MAST)
Under the hood, your function `factorial` is not stored as text. It is parsed into a **Merkle Tree**.
- The body of the function has a hash (SHA-256).
- The function definition points to that hash.
- This means if two people write the exact same logic, they generate the exact same cryptographic hash.

---

## Appendix A: Keyword Reference

| Keyword | Description | Example |
| :--- | :--- | :--- |
| `let` | Bind a variable | `(let x 10)` |
| `fn` | Define a function | `(fn add (a b) ...)` |
| `if` | Conditional logic | `(if (< x 10) ...)` |
| `while` | Loop execution | `(while (< i 10) ...)` |
| `print` | Output to console | `(print "Hello")` |
| `ask_ai`| Query LLM | `(ask_ai "Prompt")` |

---

**End of Manual.**
*Sovereign Systems &copy; 2026*
