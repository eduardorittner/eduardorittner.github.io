+++
title = 'Writing Rust simd without simd'
date = 2025-09-21T14:47:56-03:00
draft = false
+++

Rust is a modern programming language renowned for zero-cost abstractions, combining high-level ergonomics with low-level performance. However, its portable SIMD (Same Instruction Multiple Data) support remains a work in progress. SIMD instructions execute operations on multiple data elements simultaneously, amortizing decoding costs and accelerating CPU-bound tasks. Since SIMD is hardware-specific, targeting different architectures like x86 (SSE/AVX) and Arm (Neon) requires maintaing separate implementations of the same functionality, which increases the ammount of code that needs to be tested, benchmarked and maintained.

This is doable, of course, and is how things are generally done in C/C++, but one can dream. Google maintains a very good portable SIMD library for C++ called Highway, which is very similar to what Rust wants to offer. Given all this, if you want to use SIMD in your Rust code, how should you go about it? There are a couple of options

## Option 1: Getting your hands dirty with `#[cfg]`s

You can pretend it's the 90s and #ifdef your way (in Rust's case, #cfg your way) through different extensions. This obviously works, but it's a pain to test, maintain and extend. This means using the intrinsincs from the `std::arch::` modules directly, which is fine. The biggest downside with this approach (apart from what we've already said about non-portability) is that everywhere you need to use a SIMD operation, you have to `#[cfg]` it to use the appropriate extension for the current architecture. This is a tedious, manual and error-prone which is just plain unnecessary. There are of course better ways to do this, like

## Option 2: Trait-Based SIMD operations

When we're developing for different platforms, even though the specific instructions differ between architectures, they are really acomplishing the same thing. This is the perfect fit for traits, because we define an interface and decouple the interface (operations) from the implementation (arch specific SIMD instructions).

Now, the best way I know to do this is to first write your normal code as if you already all your SIMD code written, and note all the specific operations this particular code will need. We don't need to write a trait that encompasses all possible SIMD operations, we just need one that has the necessary functionality for our code. After you've noted all the operations you need, write them down into a trait, and implement that trait for all the SIMD extensions you want. This still has the same problem as the previous approach, of different implementation of the same functionality, but it's a little easier to write code for with generics, test and so on. A very cool feature with this approach is that you can implement this trait for small integer types like `u32` or `u64`, which makes it easier/faster to test the code for logic bugs etc, since the inputs now need to be smaller to hit edge cases that are more likely to happen at the boundary between loads.

The [`str_indices`](https://crates.io/crates/str_indices) crate for conversion between different indexing schemes on utf8 strings uses this technique to great effect, since it only requires a small set of SIMD operations.

## Option 3: The (unstable) [`std::simd`](https://doc.rust-lang.org/std/simd/index.html) module

Now that you've heard all about how cumbersome it is to maintain different implementations for each extension you want to target, let's talk about some actually portable options. Starting with the stdlib's `simd` module, whose opening paragraph reads:

> This module offers a portable abstraction for SIMD operations that is not bound to any particular hardware architecture.

Great, This looks like exactly what we want! But of course, there's a catch: This module is unstable and thus requires the nightly compiler. This is not a dealbreaker, especially for little side projects, but I personally prefer to stick to the stable compiler whenever possible, so I consider this a downside to this approach.

Now that you've been warned that this is an unstable module which requires the nightly compiler and may have breaking changes in the feature, but still want to go ahead and use it, how does it work? The basic SIMD type is a generic type over the element type `T` (`u8`s, `f32`s, etc.) and the number of elements `N`: `Simd<T, N>`. You can then either write code which is generic over any `Simd<T, N>` type or code written for a specific `Simd` instance. If you choose to write code generic over any `Simd<T, N>`, there's a very handy trait called `SupportedLaneCount` which is guaranteed to only be implemented for supported lane counts in the current architecture, so you get a compile error when trying to use a higher lane count than your CPU supports.

The main idea behind this module is for the user to write generic code which will then get compiled down to the most optimal instructions supported by the current cpu. So we just have one implementation which is guaranteed to compile for all targets, and be consistent between all of them, which means that we only need to test our code once. Since nothing's perfect, specially abstractions that span multiple hardware/platforms/extensions, `std::simd` prioritizes correctness (e.g., adhering to IEEE standards) and absolute consistency between targets over raw speed. While std::arch intrinsics like _mm_min_ps might be faster, they can diverge from standard behavior. Thankfully, there are escape hatches to std::arch for target-specific optimizations when absolutely necessary (which should be very rare). From the [docs](https://doc.rust-lang.org/std/simd/index.html):

> Consistency between targets is not compromised to use faster or fewer instructions. In some cases, `std::arch` will provide a faster function that has slightly different behavior than the std::simd equivalent. For example, `_mm_min_ps1` can be slightly faster than `SimdFloat::simd_min`, but does not conform to the IEEE standard also used by `f32::min`. When necessary, `Simd<T, N>` can be converted to the types provided by `std::arch` to make use of target-specific functions.

## Option 4: Community Crates

Rust also has a number of crates which have similar goals as `std::simd`, and most of them are in stable Rust. I'll write about two, but there are more which may fill other niches.

* Wide
	Wide is a crate for portable SIMD that does away with generics, instead it defines a set of wide types (like `f32x4`, `i8x16`, and so on) and common operations on those types. These operations are implemented in non-portable assembly under the hood, so you get the benefits of portable SIMD with the performance of arch-specific instructions. There are downsides to this approach, of course: Wide doesn't necessarily implement all possible SIMD operations, which means that there may be some missing functionality depending on your use case. There are also cases where wide will compile down to scalar instructions, which is also not ideal. All in all, wide is probably the simples option for portable SIMD code (which is a great compliment!) but lacks fine-grained control.

* Pulp

	Pulp is a crate inspired by Google's Highway library, which dispatches code at runtime to the best instructions supported by the current CPU. This is in a different vein to all the options previously mentioned here, which need to be compiled once for every new targeted extension. Pulp fills a very important niche in the SIMD ecosystem: runtime dispatching of architecure-specific code. This is crucial for markets like gamedev, where it's just not feasible to have either one different binary for each supported extension or have the user compile the code in their machine. Pulp also advertises portable, runtime dispatched SIMD code which is written as regular, scalar code, that is, code that is auto-vectorized. I haven't tested it extensively, and I'm not sure about the guarantees of this, since guaranteed optimizations have long been a problem of optimizing compilers, so I don't recommend it, but it's good to know that it exists.

## Conclusion

These are the main ways that I know of to use SIMD in Rust, if you feel I missed any, come talk to me! Also, if you're excited about `std::simd` being stabilized, consider taking a look at the [stabilization issue](https://github.com/rust-lang/portable-simd/issues/364).
