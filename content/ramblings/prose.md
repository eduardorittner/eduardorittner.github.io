+++
title = 'Prose Programming Language'
date = 2024-10-08T07:21:28-03:00
draft = false
+++

I am always fascinated by niche programming languages. Not the ones that are useful, but those that are fun and interesting, like Nicole Tietz's [Hurl](https://hurl.wtf/), a language based entirely on exceptions. So the other day I was thinking, and what if there was a programming language that looked a lot like text.
For example, declaring variables would be using the `is` keyword, like `a is 1`. Comparison operators would be spelled out, like `greater than`, `less than` and `equal to`. Instead of `else if` and `else`, there would be `or if` and `if not`, or something like that. `say` instead of `print`, and every expression should be a line, and blocks start with `:` and end with `.`. A small program would then be something like:

```
a is 0
while a is less than 100:
    say "counter: " a
    a is a + 1.
say "a's final values is " a
```

Just something I've been thinking about.The language itself would be a pretty simple stack based language, which is something I'm familiar with after having followed [Crafting Interpreters](https://craftinginterpreters.com/). It would be cool to have something like pattern matching which could match both on concrete instances of types as well as generic ones, like:

```
if a is
    | string "hello" -> say "a is hello".
    | number 1 -> say "a is 1".
    | bool true -> say "a is true".
    | bool false -> say "a is false".
    | number -> say "a is a number and a is not 1".
    | string -> say "a is a string and a is not \"hello\"".
```

or something like that. I like that it looks a lot like pseudo-programming, and feels very similar as to how we would explaing things to someone.

