# Thoughts on `try_trait_v2`

## Emotionally

Don't like std can do stuff I can't
Don't like extra code to work around language constraints

## Where using today

### exit_safely

### proc_macro2_diagnostic

## The 2 + 1 + 1 traits

## 2 more traits

## Simple case

3 std types

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
