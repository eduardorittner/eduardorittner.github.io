+++
title = 'Writing Rust simd without simd'
date = 2025-09-21T14:47:56-03:00
draft = true
+++

Rust is a great, modern programming language. It brought a lot of innovation to the programming space because of its focus on zero-cost abstractions, which makes for software that's as ergonomic and expressive as high-level languages like python and haskell and as performant as low-level languages like C, C++ and Zig.

Rust is also commited to backwards-compatibility, which means that all APIs in the sdtlib must be very well thought out. This process takes a long time, since they need to strike a balance between the performance and maintanability of the internals and the ease of use, expressiveness and ergonomics of the external APIs. This is very hard to do, specially for a language which doesn't target just one niche of software, but all of it. There are a lot of different use cases, requirements and constraints from different fields which must alll (ideally) fit inside one API.

One area where Rust hasn't evolved much yet is with portable SIMD (Same Instruction Multiple Data). SIMD is a term that refers to machine code instructions which execute the same operation on multiple elements at the same time. This offers a very good performance improvement, because one of the bottlenecks of CPU-intensive tasks is the decoding and pipelining of instructions. What SIMD offers is a way to operate on multiple elements with only one instruction, ammortizing the decoding cost between multiple elements instead of just one.

SIMD is great and all, but since they match CPU instructions directly by design, it's generally not very portable. In order to target x86 and arm using SIMD, you need to maintain two different implementations of the same code. That is only if you target the most widely supported extensions on each of these platrorms: SSE and Neon. The second you want to target more recent extensions the number of different implementations you have to maintain just goes up.

This is doable, of course, and is how things are generally done in C/C++, but one can dream. Google maintains a very good portable SIMD library for C++ called Highway, which is very similar to what Rust wants to offer. Given all this, if you want to use SIMD in your Rust code, how should you go about it? There are a couple of options

## Option 1: Getting your hands dirty

You can pretend it's the 90s and #ifdef you way (in Rust's case, #cfg your way) through different extensions. This obviously works, but it's a pain to test, maintain and extend. This means using the intrinsincs from the `std::arch::` modules directly, which is fine. The biggest downside with this approach (apart from what we've already said about non-portability) is that everywhere you need to use a SIMD operation, you have to #cfg it to use the appropriate extension for the current architecture.

This is a tedious, manual and error-prone which is just plain unnecessary. There are of course better ways to do this, like

## Option 2: Traits

When we're developing for different platforms, even though the specific instructions are different between architectures, they are really acomplishing the same thing. This is the perfect fit for traits, which define a set of functions, a function can then be made generic over any type that implements that trait. This decouples the interface from the implementation, which is exactly what we want.

Now, the best way I know to do this is to first write your normal code, and note all the specific operations this particular code will need. We don't need to write a trait that encompasses all possible SIMD operations, we just need one that has only the necessary functionality for our code. After you've noted all the operations you need, write them down into a trait, and implement that trait for all the extensions you want.

## Then unstable features

## Then libraries

## Then ropey's (str_indices approach)

