+++
title = 'You should write data structures in Rust'
date = 2025-10-26T10:48:46Z
draft = true
+++

Libraries are great. Reusable code is great! Everybody knows that. There's nothing worst than having to program against a badly designed API where it's not clear what's the library's responsiblity and what's the caller's responsibility. ...

Knowing how to use libraries effectively (and how to choose them) requires experience and the ability to recognize a good API from a bad API. This is an essential skill that's not really talked about that much, especially since so much of coding today involves some sort of framework, where the available choices are naturally fewer because frameworks have already made most of the choices for you.

One of the great things about software is the idea of *composability*: that you can write code once and reuse it in many different contexts. It's not an exageration to say that basically every piece of software today relies on at least some libraries, even if they are just libc. There are many great libraries which have proved to be immensely useful... [TODO]

# Why Rust is great

One of the things that makes Rust a truly great programming language is its composability. In the words of [Bryan Cantrill](https://www.youtube.com/watch?v=HgtRAbE1nBM), Rust's features make for APIs that are easy to use and hard to mess up. This is in stark contrast to the other mainstrem systems languages: C and C++, where reasoning about who owns what and what responsabilities the caller has are hard to reason about. [TODO talk about other PLs that are easy to use, but rust is great because of the borrow checker]

# Why Data Structures?

I've been talking about how libraries are great and knowing how to use (and develop) them is an invaluable skill in a programmer's toolbox. But why focus on data structures specifically, and not libraries which tackle specific problems like compression, protocols, and so on? I think data structures are great because they are generally small in scope, easy to test, but hard to get right. Data structures are also expected to be fast, which means that you can also brush up on SIMD, benchmarking and profiling, all good tools to be familiar with.

Another aspect about data structures in Rust specifically, is that in order for them to be fast and/or ergonomic, some unsafe is going to be required. Using unsafe (carefully) really made me appreciate the amount of work the Rust compiler does for you, and how the guarantees it gives you go such a long way. And figuring out the minimal amount of unsafe required to make your code work is a great challenge.

# Some examples

- [Fixed-width bit-packed vectors](https://lukefleed.xyz/posts/compressed-fixedvec/)
- [Caches in Rust](https://matklad.github.io/2022/06/11/caches-in-rust.html)
- [Learning Rust with entirely too many linked lists](https://rust-unofficial.github.io/too-many-lists/)
- [Rust collections case study: BTreeMap](https://faultlore.com/blah/rust-btree-case/)
- [Rustonomicon's Implementing Vec](https://doc.rust-lang.org/nomicon/vec/vec.html)
- [Rust Atomics and Lock's Building Our Own Channels](https://marabos.nl/atomics/building-channels.html)

# Wrapping up

Now it's your turn to go and implement your own data structures in rust (and possibly write a blog post!). I would love to get an email about your experience!

