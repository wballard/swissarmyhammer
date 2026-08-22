---
name: optionals
description: No force unwrap or IUO in non-test code, guard-let early exits in production and unwrap assertions in tests, no Optional where a default fits
---

# Swift Optionals

- **No force unwrap (`!`) in non-test code.** DON'T: `let name = user.name!`. DO: `guard let name = user.name else { … }` or `user.name ?? "Anonymous"`.
- **Use `guard let … else { return/throw }` for early exit in production code** so the happy path stays unindented. DON'T nest the whole success branch inside `if let` with a trailing `else { return }`.
- **Never `guard` in a test. Unwrap with an assertion instead.** A `guard` that returns takes the test out before its assertions run, so a broken program reads as a pass. DON'T: `guard let value = source else { return }`. DO: `let value = try XCTUnwrap(source)`, or `let value = try #require(source)` under Swift Testing. A boolean guard is `XCTAssert(condition)` or `#expect(condition)`. The same holds for a trailing `if let` that wraps the assertions of a test: the assertions never run when the binding fails, and the test still passes — write the assertion the intent asks for. This is the one place `guard` is wrong, and the reason is the same one that lets a test fixture hold an implicitly unwrapped optional: a test that cannot run must FAIL, not return.
- **No implicitly unwrapped optionals (`Type!`).** DON'T: `var session: URLSession!`. DO: a non-optional initialized in `init`, or a real `URLSession?`. Sanctioned exceptions: `@IBOutlet`, and test fixtures set in `setUp()`.
- **Don't use `Optional` where a sensible default exists.** `nil` should mean genuine absence, not error or sentinel. DON'T: `func timeout() -> Int?` that callers always `?? 30`. DO: `func timeout() -> Int { 30 }`. Keep `firstIndex(of:) -> Int?` — `nil` there is true absence, never a `-1` sentinel.
