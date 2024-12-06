+++
title = 'Fixing a helix bug'
date = 2024-10-20T11:21:43-03:00
draft = false
+++

## Introduction

I'm mostly a Neovim user, but since getting into rust I've taken up helix (mainly for rust projects for now) and it's been a great experience. I've also found that it's a lot easier to contribute to rust projects than c ones, even though I have more experience in C. I find that rust is a lot simpler to reason about locally, building projects is as simple as `cargo build` and the overall process is just smoother compared to C where you may need Make, or CMake, or Meson, or Gradle, and you also may have to manually install dependencies and on and on.

## The bug

I had already contributed to Helix's docs and was looking for issues I could participate in when I found [this issue](https://github.com/helix-editor/helix/issues/11904), which seemed pretty simple: the cursor was getting stuck on the 3th character of the string "नमस्कार"[^1], after pressing 'l' or right arrow instead of going one visual character forward it went one visual character backward.

Before doing anything I first tried to reproduce the bug on my native helix install and everything worked correctly, I then cloned the repo and built the same version as the user and sure enough the bug was there. This meant that helix was working correctly before but a regression was introduced on a recent change. Since our versions were very close, I decided to give `git bisect` a shot before diving into the code.

## Git bisect

If you're not familiar with `git bisect`, it's a very handy tool built into git that finds the first "bad" commit, in other words the commit that introduced a specific change you're looking for. I was impressed by how easy it was to start, first `git bisect start` to start the search, then you provide both a good and a bad commit through `git bisect good <commit-hash>` and `git bisect bad <commit-hash>` (if you omit the commit-hash git assumes you're talking about the current commit you're on). After that git places you on a commit between them and you have to decide whether that commit is good or bad, tell that to git `git bisect <good|bad>` and do that until you get to the first bad commit.

Before doing that, though, I took a look at the commits in the repo, looking out for any changes with regards to text and found commit [5717aa8](https://github.com/helix-editor/helix/commit/5717aa8e35b12120de067f86dbc620a6dfac91ed) which supposedly fixed an internal behavior of Helix's internal text representaion, the Rope data structure. After building this commit and realising it was also a bad commit, I narrowed the possible range down to ~80 commits, which in hindsight `git bisect` would have already done for me, but oh well.

## Git bisecting the bug

Armed with a bad commit (5717aa8) and a good commit (079f544) I started bisecting, and after 6 or 7 iterations I landed on [c754949454a6c757a41f69bb0cadee6b8fc689d7](https://github.com/helix-editor/helix/commit/c754949454a6c757a41f69bb0cadee6b8fc689d7), which is a pretty minor commit that bumped 4 dependencies: `anyhow`, `rustix`, `cc` and `unicode-segmentation`. [anyhow](https://crates.io/crates/anyhow) is an error-handling library, [rustix](https://crates.io/crates/rustix) provides safe and portable abstractions on top of POSIX-like and Unix-like syscalls, [cc](https://crates.io/crates/cc) is a tool to help compile C/C++ files inside rust projects and [unicode-segmentation](https://crates.io/crates/unicode-segmentation) provides functionality for detecting grapheme cluster, word and sentence boundaries based on Unicode.

Looking at these 4 dependencies I was pretty sure the offending change was from `unicode-segmentation`, which was bumped from `1.11` to `1.12`, so I looked at the [changes](https://github.com/unicode-rs/unicode-segmentation/compare/v1.11.0...v1.12.0) one by one and found this [suspicious commit](https://github.com/unicode-rs/unicode-segmentation/commit/8a26b3e8d3e73945f7394209b9a169e307a3e44f) which is a fix for this [issue](https://github.com/unicode-rs/unicode-segmentation/issues/125), which directly mentions the "क" character, exactly the character that Helix gets stuck on.

This is pretty convincing evidence that we've found the root cause, but just to confirm I ran the following code on unicode-segmentation versions `1.11` and `1.12`:

```rust
use unicode_segmentation::UnicodeSegmentation;

fn main() {
    let string = "नमस्कार";
    let mut g = string.graphemes(true).collect::<Vec<&str>>().into_iter();
    while let Some(c) = g.next() {
        print!("{c}, ");
    }
}
```

and as expected got two distinct results: "न, म, स्, का, र," in 1.11 and "न, म, स्का, र,"  1.12. After the upgrade, the 3rd and 4th characters are being recognized as one grapheme cluster, and this is where the bug lies, now all we have to do is dive into the code and see exactly where Helix is calling this function and how it's messing up the cursor movement.



[^1]: Funnily enough, as I was writing this blog post in Neovim, that string was also bugged and I opened this [issue](https://github.com/neovim/neovim/issues/30878).
