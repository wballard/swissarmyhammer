---
assignees:
- claude-code
depends_on:
- 01M1Y65VQ8V4TEWXDXD4Y2ZQNX
- 01M1Y5GD85944ACE2XS6SA5XWJ
position_column: todo
position_ordinal: fff780
title: Rebalance the Swift prompt rules for the toolchain gate
---
## What

The move to `swift format` changes which bullets a tool owns. One requirement takes one owner, thus each bullet must move to the correct side. Read the table of the task "Measure the Swift toolchain rules the gate needs" before you move a bullet.

Files to change:
- `builtin/validators/swift/rules/value-semantics.md`
- `builtin/validators/swift/rules/idioms.md`
- `builtin/validators/swift/rules/immutability.md`
- `builtin/validators/swift/rules/optionals.md`
- `builtin/validators/code-hygiene/rules/idioms-swift.md` — the `supersedes` key and the body section about it
- `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/swift_judgment_rules.rs`
- `crates/swissarmyhammer-validators/src/builtin/mod.rs`

### Bullets that come BACK to the prompt

| the bullet | why it returns |
|---|---|
| `value-semantics.md` mark classes `final` | the toolchain has no rule like `preferFinalClasses` |
| `idioms.md` no memberwise initializer identical to the synthesized one, for a PUBLIC initializer | measured, `UseSynthesizedInitializer` reports the internal form and is silent for the public form. Row 4 holds the evidence |

### Bullets that STAY in the prompt

| the bullet | why it stays |
|---|---|
| `idioms.md` do not repeat the type name in a static member | measured, `DontRepeatTypeInStaticProperties` reports nothing for the shape the bullet names. Row 3 holds the evidence |

### Bullets that LEAVE the prompt

| the bullet of `idioms.md` | the toolchain rule that takes it |
|---|---|
| empty-collection variables use a literal | `AlwaysUseLiteralForEmptyCollectionInit` |
| omit the return clause when it is `Void` | `NoVoidReturnOnFunctionSignature`, BOTH halves |

The empty-collection bullet holds a long note about SwiftFormat's `propertyTypes` rule and `--property-types inferred`. Delete that note with the bullet. The toolchain decides the bullet in the correct direction, thus the warning is not true.

### The `where` bullet waits for a decision

`UseWhereClausesInForLoops` contradicts `immutability.md`, which names this shape as a DON'T whose fix is `map` or `filter`, NOT a `where` clause:

```swift
for user in users { if user.isActive { names.append(user.name) } }
```

Row 5 of the measurement task states the two answers and chooses neither. Bring the evidence to a person and let them choose. Then write the answer into `idioms.md` and `immutability.md` so the two rules cannot disagree. `swift_judgment_rules.rs` holds the carve-out sentence as a test; update it for the answer.

Measured, `ReplaceForEachWithForLoop` reports `things.filter { $0 > 2 }.forEach { thing in print(thing) }` and asks for a for-in loop, which is not the `where` clause the bullet demands. Ask "does ANY enabled rule report this line, and what shape does its fix land on", and state both in the rule body.

### The three test-target requirements the gate drops

`code-hygiene/VALIDATOR.md` and `disallowed-constructs-swift.md` both state that the swiftlint rules `force_unwrapping`, `force_try` and `force_cast` stay OFF in a test target BECAUSE `idioms-swift` enables `noForceUnwrapInTests` and `noForceTryInTests`. The toolchain has neither. After the move, a force unwrap in a test file has no tool and no prompt bullet. `noGuardInTests` loses its deterministic half the same way, and `optionals.md` keeps only the words.

Decide each of the three: give the requirement to a prompt bullet, or move it to `disallowed-constructs-swift` by removing the test-target carve-out. Record the decision in `disallowed-constructs-swift.md` and in `code-hygiene/VALIDATOR.md`, because both state the old reason in words.

**Delete `idioms.md` only if it then states no bullet.** The type-name bullet, the public memberwise-initializer bullet and perhaps the `where` half all stay, thus the file is probably not empty. `idioms-swift` states `supersedes: idioms` only when the file is gone.

## Acceptance Criteria
- [ ] `value-semantics.md` states the `final` bullet again, with a DO and a DON'T.
- [ ] `idioms.md` states the type-name bullet and the public memberwise-initializer bullet.
- [ ] `idioms.md` states no bullet that a rule of the gate allowlist decides.
- [ ] `idioms.md` states nothing about `propertyTypes` or `--property-types`.
- [ ] `idioms.md` and `immutability.md` cannot disagree about a filtering loop.
- [ ] The three test-target requirements each have exactly one owner, and both documents state it.
- [ ] The `supersedes` key of `idioms-swift` agrees with what `idioms.md` holds.
- [ ] No bullet has two owners. No bullet has no owner.

## Tests
- [ ] Update `the_shipped_swift_idioms_tool_rule_owns_each_bullet_it_took` in `crates/swissarmyhammer-validators/src/builtin/mod.rs` for the new set of taken bullets.
- [ ] Write a test that holds `value-semantics.md` to stating the `final` bullet, and holds the gate silent for a class that is not `final`.
- [ ] Write a test that holds `idioms.md` to stating the type-name bullet, and holds the gate silent for `static let redColor` in `struct Color`.
- [ ] Write a test that holds `idioms.md` to naming `propertyTypes` nowhere.
- [ ] Update the carve-out test in `swift_judgment_rules.rs` for the `where` answer.
- [ ] Run `cargo nextest run -p swissarmyhammer-validators swift`. Every test passes.

## Workflow
- Use `/tdd` — write the failing tests first, then write the code that makes them pass. #swift