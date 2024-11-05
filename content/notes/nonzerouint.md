+++
title = 'Rust non zero integers'
date = 2024-11-05T09:25:45-03:00
draft = false
+++

I recently discovered Rust's [NonZero](https://doc.rust-lang.org/std/num/struct.NonZero.html) types which are implemented for all ints and uints, and as the name implies are integers which are guaranteed to not be null. This may seem a bit silly, but this one guarantee is what enables `size_of(Option<NonZero<u8>>) == size_of(NonZero<u8>)`, which basically means that an Option of a NonZero integer has no additional cost compared to an integer. This is useful whenever 0 is used as a sentinel value, since it brings that information into the type system and increases robustness, given that now 0 is not a sentinel value with the same underlying type, but a different type altogether.
