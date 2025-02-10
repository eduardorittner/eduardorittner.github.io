+++
title = "Parse, Don't Validate"
date = 2025-10-02T12:06:09-03:00
draft = false
+++

[Parse, Don't Validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/), written by Alexis King, talks about the distinction between parsing and validation. These two approaches are a means to reject invalid data in two distinct ways. Validators are pretty simple, you validate the data **after** you have constructed it. This is not necessarily wrong, but it **can** be. Parsers, on the other hand, take full advantage of type systems to ensure that they **cannot** be wrong. They do this by designing a type which can only be instantiated through a function (or functions) that validates the data.
This might not seem like much, but having distinct types for validated and non-validated data facilitates development by encoding more information about our data into the type-system. From the article:

> \[...] parsers are an incredibly powerful tool: they allow discharging checks on input up-front, right on the boundary between a program and the outside world, and once those checks have been performed, they never need to be checked again!
