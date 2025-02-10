+++
title = "Whence '\n'?"
date = 2025-10-02T12:06:09-03:00
draft = false
+++

[Whence '\n'?](https://rodarmor.com/blog/whence-newline/), by Casey Rodarmor is a very short entry on a seemingly uninteresting topic: how does `rustc` (the rust compiler) know that the binary code for `\n` is `0x0A`? Of course, we all know that `\n` is `0x0A` because it's the ASCII code for the newline character, but when looking for such a mapping in `rustc`'s source code, we find none.
The actual answer is that the ocaml compiler contains the line `'n' -> '\010'`, the ocaml compiler then compiled the first rust compiler (written in ocaml), embuing it with this knowledge. The ocaml rust compiler then compiled `rustc` and passed that knowledge on. And since `rustc` 1.0 the last version passes it on to the next `rustc` version. Deeper than you thought, huh?

From the article:
> `rustc` is currently at version 1.81.0, so this has happened at least 81 times since `rustc` 1.0 was first released, and probably many more times than that before 1.0, with `rustc`s furtively smuggling `0x0A` bytes from one to the other, all the way back to when it was written in OCaml, when finally the first `0x0A` byte was stuffed into a `rustc` binary by the OCaml compiler, which evaluated it from a decimal character escape `'\010'`.

This is closely related to the behavior presented in Ken Thompson's famous Turing Award Lecture [Reflections on Trusting Trust](https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ReflectionsonTrustingTrust.pdf), just not as dangerous.
