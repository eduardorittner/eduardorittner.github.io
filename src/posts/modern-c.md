+++
title = 'Writing C in 2025'
date = 2025-01-20T11:21:43-03:00
draft = false
+++

# Why?


# Editing tools

Tools are made to make your life easier, take advantage of that.

## [Clangd](https://clangd.llvm.org/) (LSP)

There are a handful of C language servers, but clangd is currently the best. It provides all the niceties of a language server while being resonably fast. Nowadays most code editors have support for lsp, and setting them up is usually pretty simple.

## [Clang-format](https://clang.llvm.org/docs/ClangFormat.html) (Formatter)

Having a tool that formats code in a consistent way throughout your code-base is good because it frees up mental space for other, more important things while keeping the code very readable. I find the defaults for clang-format are pretty good, but it's endlessly [customizable](https://clang.llvm.org/docs/ClangFormatStyleOptions.html), and you can even [disable clang-format on a piece of code](https://clang.llvm.org/docs/ClangFormatStyleOptions.html#disabling-formatting-on-a-piece-of-code) when you want them formatted in a very specific way.

# Compiler

Of course when working with C you'll be using a compiler, try to work with it as much as possible, the compiler is your friend.

## Don't use compiler-specific extensions (unless absolutely necessary)

They make your code less portable, while generally providing little to no benefit. The most dangerous compiler extensions are the ones that affect control-flow, such as GCC's [cleanup attribute](https://gcc.gnu.org/onlinedocs/gcc/Common-Variable-Attributes.html#index-cleanup-variable-attribute), since your code will be outright wrong when using any other compiler that doesn't implement this/

## Always compile with all warnings

Compiler warnings are very helpful and have little to none false positives. Recommended flags are:

- `-Wall`: Enables all standard checks
- `-Wextra`: Enables some more checks
- `-Wpedantic`: Warns about non-standard or obsolete C constructs. Encourages more portable code
- `-Wformat=2`: Warns about incorrect string formatting in functions like `printf` and `scanf`.
- `-Wconversion`: Warns about all non-explicit type conversions

## Use the most recent available standard

Unless you're developing for an embedded environment which doesn't have the latest C compiler, there's no reason to be stuck on C99, so just use the latest standard available. C11 is much better than C99, which is much better than C89. Always write to a specific standard, as in `std=c11`, don't leave it unspecified.
# Code style

These rules are a meant to make your code simpler, easier to reason about, and avoid constructs or patterns which (can) lead to errors, bugs, and non-intended behaviors.

## Use `bool`s, not `int`s

C has had a dedicated boolean type for quite some time, so use it. It communicates intent and signals clearly that there are only two valid conditions.

## Avoid `unsigned` integer types

C has a bunch of weird conversion rules that spring out of nowhere and sprinkle UB all over your program. For example, any `unsigned + signed` operation converts the `signed` value to `unsigned`, which may trigger UB when the `signed` value is negative and cannot be represented as unsigned. An exception to this rule is when dealing with pointers, use the dedicated types like `ptrdiff_t`.
## Always use include guards for headers

Include guards let you include a header twice without breaking compilation. Doing any kind of development without them is just a hassle and more trouble than it's worth.

```c
#ifndef MYFILE_H
#define MYFILE_H

...

#endif MYFYLE_H // ifndef MYFILE_H
```

## Always comment `#endif`s of large sections

As illustrated in the example above. Some macro conditional sections can be very large (sometimes spanning the whole file), commenting the equivalent `#if` on the `#enfid` helps situate the reader.

## Use `const` liberally

`const` specifies to the compiler that a variable shouldn't change. It may open up some optimizations, but it also enforces code correctness, since any `const` violation will be caught by the compiler. Note that there are some caveats to this: for non-pointer types, use `const` only for local variables, not for function parameters (since they are already passed by value). For pointer types in function parameters, generally what you want is for the *pointee* to be `const`, not the pointer, so pay attention to that.

## Use `static`

The `static` keyword is equivalent to saying something is local, i.e. it can only be accessed from the same file. The general advice is that everything you declared should be `static` by default unless there is a good reason for it not to. This is more important for libraries than binaries, but using `static` makes the overall namespace cleaner and less cluttered.

## Use anonymous structs for sum-types

## Document and assert code invariants

- Avoid any unsigned types (conversion rules are bonkers and they don't have guardrails)
- For function-specific macros, #define them at function start and #undef them at function end
- Use sizeof for variables, not types
- Document code invariants, and assert them whenever possible
- Use anonymous structs for sum-types
- Use structs for optional arguments [https://github.com/mcinglis/c-style?tab=readme-ov-file#use-structs-to-name-functions-optional-arguments]


- Use Doxygen for documenting code
- Better assert [https://codeberg.org/NRK/libz1/src/branch/zattr]

# Safety tools

- CPPcheck
- Clang-tidy
- UBSan, ASan, etc. (clang and gcc)
- Valgrind

# References

[c-style](https://github.com/mcinglis/c-style)
[c static analyzers](https://nrk.neocities.org/articles/c-static-analyzers)
