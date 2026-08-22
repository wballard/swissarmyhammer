---
name: preconditions
description: guard at the head of a production scope, assertionFailure plus a production log for a recoverable surprise
---

# Swift Preconditions

- **Validate a precondition with `guard`, at the head of a production scope.** `guard` states the requirement where a reader looks for it and leaves the happy path unindented; a nested `if` hides the requirement inside the branch it protects. `optionals.md` states this for an optional binding, and states why a test is the one place `guard` is wrong; this bullet reads every other precondition — a Boolean test, a count, a state check. DON'T: `func send(_ message: String) { if isConnected { if !message.isEmpty { transport.write(message) } } }`. DO: `guard isConnected else { return }`, then `guard !message.isEmpty else { return }`, then `transport.write(message)`.
- **An unexpected but recoverable condition gets `assertionFailure` AND a production log.** The assertion stops the debug build, so the author sees the surprise; the log records it in release, where the assertion is gone. Silence hides the defect from everybody. This reads the condition the code did not expect — a lookup that should always find its key. A requirement the caller is expected to miss is the bullet above, and its bare `else { return }` is right there. DON'T: `guard let found = table[key] else { return "" }`, which records nothing. DO: `guard let found = table[key] else { assertionFailure("no value for \(key)"); logger.error("settings key is missing; the empty default stands"); return "" }`. That `""` is the default the log names, not a sentinel that hides a reason. `error-handling.md` owns `fatalError`/`preconditionFailure` whole, so this bullet names neither.
