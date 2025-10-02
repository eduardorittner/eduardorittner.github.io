+++
title = 'Expression parsing with SIMD'
date = 2025-10-02T03:15:38Z
draft = false
+++

SIMD is great for operations on data with no interdependencies, which is quite the opposite from parsing mathematical expressions, because they require context, a token's meaning may depend on tokens that are far away from it. Knowing this, wouldn't it be fun to be aple to have a SIMD parser? Here's what I've thought of so far:

1. Tokens should be lexed into a `TokenId` and `Token`, the `TokenId` would be a `u8` or `u16` which points to the `Token` value. Since the value is only actually necessary for numbers, there could be some set of predefined values of `TokenId`s which represent the operators. Then, during parsing, since we only care about the token "type" (whether it's a number or operator), we could just operate on `TokenId`s and only during the interpretation of the parsed expression we would access the `Token` values using the equivalent `TokenId`s.
2. Parsing can be done similar to Pratt-style parsers, thinking of precedence first. The way this could be done is for example, we do one pass for every operation, in the order of higher to lower precedence. So we would do one pass for multiplication, then mask out all the adjacent tokens, so they don't get bound to lower precedence operators. This would have the unfortunate consequence that multiplication and division would not have the same precedence, but it's fine.
3. One thing I still haven't figured out yet is what would be the best output for the parser: some type of AST (I don't think so), maybe something like RPN? Not sure.

Since this would only be an experiment, it'd be fine to have some limitations, for example having `TokenId`s be `u8`s probably means that an expression can't be longer than 256 tokens (less than that because of the reserved values for operators), which makes a SIMD parser pointless to start with, but whatever.
