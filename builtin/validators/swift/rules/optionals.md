---
name: optionals
description: guard-let early exits in production and unwrap assertions in tests, no Optional where a default fits
---

# Swift Optionals

- **Use `guard let … else { return/throw }` for early exit in production code** so the happy path stays unindented. DON'T nest the whole success branch inside `if let` with a trailing `else { return }`.
- **Never `guard` in a test. Unwrap with an assertion instead.** A `guard` that returns takes the test out before its assertions run, so a broken program reads as a pass. DON'T: `guard let value = source else { return }`. DO: `let value = try XCTUnwrap(source)`, or `let value = try #require(source)` under Swift Testing. A boolean guard is `XCTAssert(condition)` or `#expect(condition)`. The same holds for a trailing `if let` that wraps the assertions of a test: the assertions never run when the binding fails, and the test still passes — write the assertion the intent asks for. This is the one place `guard` is wrong, and the reason is that a test that cannot run must FAIL, not return.
- **Don't use `Optional` where a sensible default exists.** `nil` should mean genuine absence, not error or sentinel. DON'T: `func timeout() -> Int?` that callers always `?? 30`. DO: `func timeout() -> Int { 30 }`. Keep `firstIndex(of:) -> Int?` — `nil` there is true absence, never a `-1` sentinel.
