---
assignees:
- claude-code
position_column: todo
position_ordinal: ffed80
project: swift-validator
title: 'swift partial: prefer `swift package format --lint` when the Airbnb plugin is a dependency'
---
`builtin/_partials/project-types/swift.md:40-41` names three tools loosely and commits to none:

```
- Format: `swift format -i -r Sources Tests` (or `swiftformat .`) — run before committing
- Lint: `swiftlint` (if present)
```

`swift format` and `swiftformat` are DIFFERENT TOOLS. The first is Apple's `swift-format`, shipped with the toolchain. The second is Nick Lockwood's SwiftFormat, installed separately, with a different config format and a different rule set. Offering them as interchangeable alternatives in one line is wrong and will send an agent to the wrong config file.

## Two fixes

1. Split the two tools onto their own lines. Say which config file each reads: `swift format` reads `.swift-format`; `swiftformat` reads `.swiftformat`.

2. Detect the Airbnb Swift Package Manager plugin and prefer it when present. A package that declares

   ```swift
   .package(url: "https://github.com/airbnb/swift", from: "1.0.0")
   ```

   gets one command that runs SwiftFormat AND SwiftLint with a shared config and one exit code:

   ```
   swift package format --lint          # check, do not write
   swift package --allow-writing-to-package-directory format   # non-interactive fix
   ```

   Useful switches: `--paths Sources Tests Package.swift`, `--targets <Target>`, `--exclude Tests`, `--swift-version 6.2`.

   Exit codes: non-zero on any failure in `--lint` mode; in fix mode, non-zero only for SwiftLint rules, because those cannot autocorrect.

   For a project with this dependency, one install and one command beats wiring two tools ourselves.

## Related

Card ^7fgdenq — "guidelines: swift format instruction — honor an existing .swift-format/.swiftformat config, defaults otherwise" — covers the config-detection half. Read it before starting. These two cards may merge; if they do, close one and say so.

## Acceptance

- The partial names each tool once, with its own config file.
- The plugin path is documented with the real command lines above.
- No line implies `swift format` and `swiftformat` are the same tool. #tool-validators