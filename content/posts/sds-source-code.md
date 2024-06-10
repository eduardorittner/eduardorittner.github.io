+++
title = 'Reading C Source Code: SDS'
date = 2024-02-26T10:48:46Z
draft = false
+++

Reading source code can be a great way to learn from those who have more experience, as well as see how the projects are structured, how functions are written, etc. So today we are going to be diving in the sds source code. Sds stands for simple dynamic strings, and it's a three file library for dealing with dynamic strings in c, most notably, it's used inside Redis for everything pertaining to strings.

# Strings in c

For starters, let's first see how strings work in c, and why a custom library is deemed necessary when working with them. Strings in c are nothing more than arrays of chars, terminated by a null character '\0'. One obvious consequence is that, to know the length of a string you must traverse it in full everytime. Another not so obvious consequence (but arguably more important) is that many c functions in the standard library expect the null terminator, and that can lead to some simple bugs (at best) or severe security vulnerabilities (at worst).

# Simple dynamic strings

Sds is a library intended to replace the c builtin strings entirely, while being mostly compatible, so you can pass an sds string to any c function that expects a char*. How they work is that the header (containing useful information such as memory allocated, length of string, etc.) is stored directly before the string array. That way you can pass around a char* to sds functions, and if the want to get the string's length, the just decrement the pointer and get the information. This also means that you can pass the char* to any c function expecting a regular string, since sds strings are also null terminated.

# Implementation

Now let's look at some [https://github.com/antirez/sds](source code)! First, the header files:

## sdsalloc.h

This is a simple file for defining the allocator to be used for sds strings, it makes it trivially easy to switch from malloc to jmalloc, for example, or any other memory allocator. Another important point is that by using s_malloc and friends, a codebase can use 2 different allocators, one for sds strings, and any other for the rest of the code.

## sds.h

This file contains the definitions of the header structs where the length and allocated memory are stored. There are 5 header types, each one able to store a certain number of bits of length:
- 5 bits
- 1 byte
- 2 bytes
- 4 bytes
- 8 bytes
An important thing to note is that they are defined with ```__attribute__((__packed__))```, which means that the struct will not have any padding inside the struct. In C, structs receive the same alignment as the member with highest alignment inside it, so for example, the following struct
```
struct thing {
    int32_t length;
    int32_t alloc;
    char c;
};
```
will have sizeof == 12, not 9, because the type int32_t is 4 byte alligned, then the struct will receive 3 unused bytes of padding to make it 4 byte aligned as well. The attribute packed is a way to tell the compiler to not insert any padding and not align the struct to its highest alignment member.
Another thing to note about the header is that all of them contain an unsigned char for flags, this is necessary to be able to distinguish between different types of headers, since different headers have different sizes. Therefore, to find the type of header of an sds string, you decrement its char*, obtain the header size from that, and then decrement the appropriate amount to access the header information.

Aside from the header struct declarations and a few convenience macros, there are 6 function implementations, all of them `static inline`. I'm not exactly sure what that does at the compiler level, but I think it's a way to force these functions to be inlined wherever they are called, as opposed to it being a "suggestion" like the `inline` keyword is.
These 6 functions are the only ones that deal directly with the header information, all other functions in sds.c deal with the string content in the sds strings, and whenever they need something from the header they call one of the functions from sds.h.

## sds.c

In this file are located most of the string functions one could possibly need, including creating, modiyfing, duplicating, formatting and destroying sds strings. Most (if not all) of the functions defined in `<string.h>` have an equivalent implementation in this file.
