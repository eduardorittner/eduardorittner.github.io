+++
title = 'Tips on faster rust compile times'
date = 2024-08-20T00:51:11Z
draft = false
+++

Building a rust project can take quite a while (especially on my old laptop) and it's always nice to have a few tricks up your sleeve to mitigate these long compile times, so here are some tips in no particular order:

# Linking libraries

If your code uses any libraries which may (or can be) already installed on your system, there most likely is a way for you to tell cargo not to compile that library and just use the one you have instead. Note that this can sometimes go wrong if the library version you have is incompatible with the one required by cargo, so be aware of that.
The specific way to do this will vary based on the crate you're using, some will be explictly documented, and others won't, so you may have to look into their build.rs code, but what it boils down to is running `env YOUR_LIBRARIES_ENVVAR=1 cargo build` with the envvars you've found, for example, the [ztsd-sys](https://crates.io/crates/zstd-sys) doesn't tell you how to load it, but if you look into its [build.rs](https://github.com/gyscos/zstd-rs/blob/main/zstd-safe/zstd-sys/build.rs) it checks for a "ZSTD_SYS_USE_PKG_CONFIG" env var, so you would run `env ZSTD_SYS_USE_PKG_CONFIG=1 cargo build`. Other crates may have some type of feature you can set or unset to toggle compilation or linking, such as [libsqlite3's](https://crates.io/crates/libsqlite3-sys) "bundled" feature.
Either way, linking libraries instead of compiling than can lead to a minor improvement in overall compile times, and in one example of this, while compiling wezterm I was able to reduce total (cold) compilation times by 5 minutes (of total 21, so a 25% improvement for free). This case is likely an outlier due to wezterm relying on 4 libraries I was able to link instead of compile, and my laptop having only 2 cores, on a machine with 4 or more the gains would've probably been much smaller, but a win is always a win.

