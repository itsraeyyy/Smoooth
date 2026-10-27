---
name: rust-best-practices
description: >-
  Write idiomatic, robust, maintainable Rust. Use this skill whenever you write, review,
  refactor, debug, or explain Rust code: new crates, modules, functions, structs, traits,
  error types, async code, unsafe or FFI code, Cargo.toml, tests, or rustdoc. Trigger on
  any mention of Rust, cargo, crates, .rs files, the borrow checker, lifetimes, traits,
  clippy, or rustfmt, even when the user does not ask for "best practices" or "idiomatic"
  code. Synthesized from the Google Rust Style Guide, the Rust API Guidelines,
  Effective Rust, Rust Design Patterns, and Microsoft's Pragmatic Rust Guidelines.
---

# Rust Best Practices

Distilled from Google Rust Style Guide, Rust API Guidelines, Effective Rust, Rust Design Patterns, and Microsoft Pragmatic Rust Guidelines.

## 1. Mindset

- **Write Rust-shaped code.** Don't port C++, Java, C#, or Python constructs one-to-one.
  Model errors with `Result`, resources with ownership and `Drop`, polymorphism with
  traits and enums, and nullability with `Option`.
- **Make invalid states unrepresentable.** Push invariants into types: enums instead of
  bool flags, newtypes for units and IDs, fallible constructors that validate once.
- **Application code is not library code.** Library code follows strict API rules. Application
  code should favor directness, simplicity, concrete types, and `anyhow`-style error contexts.
- **Let the tools do the work.** `cargo fmt` owns formatting, `cargo clippy -- -D warnings`
  owns idiom checks, and the compiler owns memory safety.

## 2. Verification Protocol

Run these before delivering code changes:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

- Fix lints directly rather than silencing them.
- Avoid `#![deny(warnings)]` in source headers; enforce strict warnings in CI with `-D warnings`.

## 3. Ownership and Borrowing

- **Prefer unique ownership.** Reach for `Rc`/`Arc` only when shared ownership is genuinely
  part of the design, and `Mutex`/`RwLock` only when interior mutability is required across threads.
- **Do not clone to appease the borrow checker.** Avoid unnecessary `.clone()` calls:
  - Take `&str` instead of `&String`, `&[T]` instead of `&Vec<T>`, `&T` instead of `&Box<T>`.
  - In fallible lookups (e.g. `serde_json::Value` or `serde_yaml::Value`), query with `&str` directly
    instead of allocating temporary `Value::String(...)` instances.
  - Move fields out of `&mut self` with `std::mem::take` or `Option::take` rather than cloning.
- **Take ownership by value when storing.** If a function consumes or stores an argument, take `T`
  by value rather than `&T` followed by an internal `.clone()`.

## 4. Types and API Design

- **No boolean blindness in public/shared signatures.** Avoid ambiguous boolean parameters like
  `process(true, false)`. Use explicit two-variant enums or dedicated methods.
- **Derive standard traits eagerly:** `Debug` on all public types, `Clone`, `Default`, `PartialEq`, `Eq`.
- **Prefer `From` / `TryFrom`** over raw `as` casts to prevent silent truncation or sign loss.
- **Constructors:** Provide `new()` for default instantiation, and builder patterns (`FooBuilder`)
  when configuring more than two optional parameters.

## 5. Errors and Panics

- **Panics are not exceptions.** Never use `unwrap()` in production library or server code.
- Every `expect()` must document the invariant why it cannot fail (`.expect("mutex poisoned")`).
- In functions returning `Result`, propagate errors with `?` rather than panicking.
- Use `thiserror` for structured domain/library error types and `anyhow` for application orchestration.

## 6. Performance & Idioms

- Use `sort_by_key(|x| std::cmp::Reverse(x.timestamp))` instead of verbose comparator closures.
- Use pattern slices in `replace([' ', '_'], "-")` rather than chained single-character replaces.
- Avoid nested `format!` macros; pass formatted expressions directly into outer format strings.
