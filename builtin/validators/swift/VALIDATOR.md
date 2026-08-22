---
name: swift
description: >-
  Swift review guidelines from Apple's API Design Guidelines and Apple's
  open-source libraries (stdlib, swift-nio, swift-argument-parser,
  swift-collections, swift-format) — casing, naming clarity, doc parameter
  naming, fluent usage, idioms, value semantics, access control, error
  handling, optionals, concurrency, and state modeling applied to changed
  Swift files.
metadata:
  version: "{{version}}"
match:
  files:
    - "**/*.swift"
---

# Swift Review Validator

Language-scoped review guidance for changed Swift (`.swift`) files, grounded in
two sources: Apple's **Swift API Design Guidelines** and the idioms of Apple's
own **open-source Swift** projects.

**If a tool can decide it, the tool owns it.** This set holds the questions a
reader must JUDGE, and it states none a linter answers. Two tool rules of
`builtin/validators/code-hygiene/` carry the deterministic half: `idioms-swift`
runs swiftformat over every changed Swift file, and `disallowed-constructs-swift`
runs swiftlint. Ten bullets that stood here are theirs now, and half of an
eleventh; each rule body names what it took and carries the measurement
behind each.

Neither declares a `supersedes` key. That key names a WHOLE prompt rule, and
each gate decides BULLETS spread across several rules of this set, so naming
one would take its remaining bullets out of every review.

One requirement takes one owner, and a bullet is split at the requirement: a
bullet stating two takes the tool as owner of the one the tool decides. A
bullet stating ONE requirement the tool reads only partly stays here whole, and
says which part the tool misses. Where a tool's finding needs an exception, the
author writes that tool's own inline directive with the reason after it — never
a rule here.

Each rule here is an **in-file idiom judgment** read from the diff; there are no
engine probes on this side. Every rule that fires must be fixed — review is
binary pass/fail, with no advisory or severity tier among findings. Only add a
rule if you want it enforced. Formatting-only concerns (whitespace, indentation,
import ordering, semicolons) belong to `swift-format`. Every rule reads plain
Swift, so none opens with a detection clause.
