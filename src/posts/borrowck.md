+++
title = 'Fighting the Borrow Checker'
date = 2024-10-03T12:06:09-03:00
draft = true
+++

# Introduction

I've been going through Thorsten Ball's [Crafting Interpreters](https://craftinginterpreters.com/) and implementing Lox in rust. It's been going great so far, and I'm enjoying doing things in rust, some things are a lot simpler and easier than what he does in C, and I've sprinkled some lifetimes here or there in the codebase to great success. However, I'm having some trouble right now, in trying to implement functions. For some context, Lox is a dinamically typed language, so every value in lox is actually the same Value type, which I've implemented as an enum like so:

```rust
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    Nil,
}
```

We can see that each value is an enum variant with a payload, which is simple enough. However the real issue starts when we introduce functions, which are also first-class values. Implementing them might seem simple enough, just add another `Function(FunStruct)` type where `FunStruct` defines a function, but it's not that simple. The function struct is defined as

```rust
pub struct FunctionStruct {
    name: String,
    code: Chunk,
    other_stuff: int32,
}
```

where `Chunk` is a struct which holds the compiled bytecode and associated constants. The problem lies in that a `Chunk` has a `Vec<Value>` which means it can hold a `FunctionStruct` which in turn references some other `Chunk` or itself. All this to say that, with the current implementation `Chunk` is now a self-referential struct, which is something notably hard to pull off in rust. I've been trying to brute-force my way into a solution by just fixing compiler errors one by one for days now, but it looks like it's not going anywhere, so I decided to write about the problem and see if that helps.

## Code specifics

Before looking for a solution, I'll first define how the compiler works as of now, and from there see whether there are some refactors which could solve this. The compiler is pretty simple and has 3 major constructs: `Vm`, `Parser` (which is actually a parser + compiler, so from now on I'll refer to it as `Compiler` in order to make things simpler) and `Lexer`. `Vm` is the virtual machine which actually runs the compiled bytecode, and it has a `code: Chunk` field along with other things it needs to run the code, such as a stack, a hashmap for global variables and a formatter. `Compiler` and `Lexer` is where things start to get interesting, their definitions are:

```rust
pub struct Compiler<'a> {
    lexer: Lexer<'a>,
    source: &'a str,
    code: &'a mut Chunk,
    rules: HashMap<TokenKind, ParseRule<'a>>,
    scope: Scope<'a>,
}

pub struct Lexer<'a> {
    source: &'a str,
    rest: &'a str,
    offset: usize,
    peeked: Option<Result<Token<'a>, miette::Error>>,
}
```

The `Lexer` contains two references to the source code: one that points to the entire thing, and another that points to the portion of the source code which has not yet been consumed. The `peeked` field allows tokens to be peeked but not consumed right away, which is needed for lookahead in the compiler. `Compiler` in turn contains a `lexer` as well as its own reference to the source code and a `Chunk` where it's storing the compiled bytecode.

# The Objective

In the book, functions are implemented in a way where every `FunctionStruct` object contains a (mutable) reference to its compiled bytecode, and everytime a `Compiler` encounters a function declaration, a new nested `Compiler` is instatiated, which then compiles the function body and returns a `FunctionStruct` value to be stored in the "father" `Compiler's` constant table. Note that this works for function declarations inside other functions, creating `Compilers` recursively as needed to compile inner functions. Function calls are much simpler, the parameters are placed on the stack on top of the `FunctionStruct` value, and when the function call opcode is encountered, the vm simply adds a new `CallFrame` to the call stack with the `FunctionStruct` value and execution resumes from there. I would like to be able to implement something very similar to this, in order to have the same semantics as the book's compiler and language. 

# Requirements

So basically what we want is to have a mutable reference to `FunctionStruct` only when we're compiling its associated function, and after that all we have are immutable references to it, in `Chunk`, `Compiler`, `CallFrame` and the like. `FunctionStruct` can own its `Chunk`, and wherever we need a `Chunk` we can simply have a `FunctionStruct` instead, which should already resolve the referential-struct situation.

# Just Do It

After having the requirements clear in my head, I went straight into the codebase changing the plumbing necessary, every `Chunk` field was replaced with a `FunctionStruct` field, and some helper methods were implemented to make things easier. Now that the code is working correctly with `CallFrame`s and `FunctionStruct`s (with no functions, that is) we can focus on function support for the `Compiler`.
