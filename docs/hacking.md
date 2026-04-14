# Thoughts on `try_trait_v2`

Let me start off by saying, I _love_ `trait Try` and really hope to see it accepted soon. I'm happy (and starting) to help with moving that forwards too.

## Emotionally

I like the trait for two reasons:

1. I don't like it when std can do stuff I can't, `Try` opens up the power and versatility of `?` to my own code.
1. I don't like writing extra code when it feels like I'm just working _around_ language constraints. `Try` lets me add `impl`s directly to a custom `Result` type or flatten nested contructs inside `Option`s & `Result`s

## My related crates

To date I've thought about using the trait multiple times but always found I would end up with more code than simply working around `Option` & `Result`. Then  I ran into a case where I **absolutely had to** add a trait to a `Result` - I wanted something to return from `fn main()` which gave me control over exit codes, ensured `Drop` was run properly _and_ didn't leave me with go-like error handling in `main` :feelsgood:

### exit_safely

[MusicalNinjaDad/exit_safely](https://github.com/MusicalNinjaDad/exit_safely) works with derived Try implementations via [MusicalNinjaDad/try_v2](https://github.com/MusicalNinjaDad/try_v2) to solve the problem of returning from main with Drop and control over exit codes.

### proc_macro2_diagnostic

[MusicalNinjaDad/proc_macro2_diagnostic](https://github.com/MusicalNinjaDad/proc_macro2_diagnostic) brings `?` to compiler diagnostics for proc macros.

### try_v2

[MusicalNinjaDad/try_v2](https://github.com/MusicalNinjaDad/try_v2) provides a set of derive macros to make `Try` more accessible. (See below for details)

## Criticism: complexity

After working with the trait in various use cases, taking it apart to try (!) and derive a generic implementation and spending time reading RFCs, unstable books, comments in std source code, github issues, PRs, discord discussions, ... to my mind, the remaining complexity in `Try` is **inherent**. The implementation is as simple as possible to provide the power and flexibility required for the more meaningful use cases.

## The 3 traits + 1 type + 1 function

When talking about `Try` below, I will usually consider the following traits in one package:

- `trait Try` (`try_trait_v2`)
- `trait FromResidual` (`try_trait_v2`)
- `trait Residual` (`try_trait_v2_residual`)
- `type !` (`never_type`)
- `fn try_collect()` (`iterator_try_collect`)

### 2 more experimental features

As wierd as it may be from the naming `try_blocks`, `try_blocks_heterogeneous` are more separate from a usage point of view.

## Simple case

3 std types + Poll?

### Flattening nested types

### Flattening trait MyFunctionsExt

### Boilerplate -> Derive

- traits themselves
- all the nice functions that std lib have in common

### Gotchas -> Derive

- choice of residual
- interconversion with result, overlapping Into impls
- &! not infallible

## Complex cases

### struct with hidden inner

### Box vs Vec vs Option

### ? with sideeffects

- global state inherently evil
- diagnosticresult
- loggedresult
- async & channels
