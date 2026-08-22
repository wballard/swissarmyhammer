---
name: idioms
description: Literal empty collections, redundant Void return clauses, type-name repetition in static members
---

# Swift Idioms

Semantic idioms that read wrong in a diff. (Whitespace, indentation, and import
ordering are `swift-format`'s job, not review findings.)

- **Empty-collection variables use a literal with a type annotation, not a call.** DO: `var items: [Int] = []`. DON'T: `var items = [Int]()`. This includes `Set` and every other `ExpressibleByArrayLiteral`/`ExpressibleByDictionaryLiteral` type: `var ids: Set<String> = []` is the idiomatic form — do NOT flag it toward `Set<String>()`; the annotated literal wins, and flip-flopping between the two forms across review rounds is always a validator error. SwiftFormat's `propertyTypes` rule under `--property-types inferred` rewrites this bullet's DO into its DON'T — measured on 0.62.1, `var items: [Int] = []` becomes `var items = [Int]()` and `var ids: Set<String> = []` becomes `var ids = Set<String>()`. The `idioms-swift` tool rule therefore enables neither the rule nor the option, and its body carries the measurement. Do not add either one.
- **Omit the return clause entirely when it is `Void`.** DON'T: `func f() -> Void {}`. DO: `func f() {}`.
- **Don't repeat the enclosing type's name in a static member.** DON'T: `static let redColor` on `Color`. DO: `static let red`.
