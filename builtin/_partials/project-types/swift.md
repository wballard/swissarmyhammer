---
title: Swift Project Guidelines
description: Best practices and tooling for Swift projects (SwiftPM and Xcode)
partial: true
---

### Swift Project Guidelines

**Project default — prefer ULID for unique identifiers:** use [yaslab/ULID.swift](https://github.com/yaslab/ULID.swift). Prefer ULID over UUID for new identifiers — ULIDs are lexicographically sortable and time-ordered.

Add it to `Package.swift` dependencies:

```swift
dependencies: [
    .package(url: "https://github.com/yaslab/ULID.swift", from: "1.2.0"),
],
targets: [
    .target(name: "MyTarget", dependencies: [
        .product(name: "ULID", package: "ULID.swift"),
    ]),
]
```

Usage:

```swift
import ULID

let id = ULID()
```

**Testing — do NOT glob; the test runner discovers tests automatically:**
- SwiftPM all: `swift test`
- SwiftPM single: `swift test --filter '<TestTarget>.<Suite>/<test>'` — `--filter` takes a regular expression, and the target name is part of it
- Xcode: `xcodebuild test -scheme <Scheme> -destination 'platform=macOS'`

**Common commands:**
- Build: `swift build` (SwiftPM); `xcodebuild build -scheme <Scheme>` (Xcode)
- Run: `swift run`
- Deps: edit `Package.swift`; `swift package resolve`, `swift package update`

**Build directory hygiene — reset when you start a task.**

```
swift package reset
```

`.build/` gets larger with each build. It holds the object files of every dependency, every target and every earlier configuration, and it is never trimmed. On a package you work on for a long time, the directory can fill the disk.

Run `swift package reset` when you start a new task. It removes the full build directory and the resolved-dependency state. The next `swift build` or `swift test` resolves the dependencies again and builds from the start, so it is slow — do this once at the start of a task, not between builds inside one task.

`swift package clean` removes only the build artifacts and keeps the dependency checkouts. Use `clean` for a rebuild inside a task. Use `reset` to get the disk space back.

For Xcode builds the same files accumulate in `~/Library/Developer/Xcode/DerivedData/`. Remove the DerivedData directory of the project when the disk gets full.

**Formatting and linting.** Format before you commit.

`swift format` and `swiftformat` are DIFFERENT programs. Each one reads its own config file and applies its own rules:

| tool | command | config file it reads |
|---|---|---|
| Apple swift-format | `swift format` (in the toolchain) | `.swift-format` (JSON) |
| SwiftFormat (Nick Lockwood) | `swiftformat` | `.swiftformat` |
| SwiftLint | `swiftlint` | `.swiftlint.yml` |

Use the tool whose config file the repo already holds, and obey that config. Do not write a config file as a side effect of formatting. Do not add style flags that disagree with a config file that is already there. When the repo holds no config file, use the tool defaults.

**Prefer the Airbnb plugin when `Package.swift` declares it.** Look for this dependency:

```swift
.package(url: "https://github.com/airbnb/swift", from: "1.0.0"),
```

It gives one command that runs SwiftFormat AND SwiftLint, with a shared config and one exit code:

```
# check — the run writes no file
swift package --allow-writing-to-package-directory format --lint

# fix — non-interactive
swift package --allow-writing-to-package-directory format
```

Give `--allow-writing-to-package-directory` to the check run also. The plugin always asks for write permission, so a bare `swift package format --lint` stops with a permission error in a non-interactive shell. The check run still writes no file.

Switches: `--paths Sources Tests Package.swift`, `--target <Target>` (singular — `--targets` is not a switch and exits non-zero), `--exclude Tests`, `--swift-version 6.2`.

Exit codes: `--lint` exits non-zero on any failure. Fix mode exits non-zero only for a SwiftLint rule, because SwiftLint cannot correct those automatically.

One caution about the plugin. Its config sets `--property-types inferred`, so fix mode rewrites `var items: [Int] = []` into `var items = [Int]()`. The `idioms` review rule wants the annotated literal. Put the annotated form back after a fix run.

**Without the plugin**, name the tool and its options:

- SwiftFormat, 0.62.1 or newer:

  ```
  swiftformat . --pattern-let inline --short-optionals always \
    --single-line-for-each convert --guard-like-if-statements convert
  ```

  State those four options. Their defaults are `hoist`, `preserve-struct-inits`, `ignore` and `preserve`, and a bare `swiftformat .` writes code that the `idioms-swift` review gate then reports. A repo `.swiftformat` that already states the four options makes a bare `swiftformat .` correct — the tool finds that file in the file's own directory or a parent, and obeys it. Add `--lint` to check without writing.

- Apple swift-format: `swift format -i -r Sources Tests` to write, `swift format lint -s -r Sources Tests` to check. The tool finds `.swift-format` in the file's own directory or a parent, and obeys it. Keep `-s` on a check run — without it `swift format lint` reports warnings and still exits 0.

- SwiftLint: `swiftlint` to report, `swiftlint --fix` to correct what it can.

**Write the project Swift version in `.swift-version`.** SwiftFormat reads that file, and several rules stay silent without it — they will not suggest an API the toolchain may lack. `Package.swift` is not read for this.

**File locations:** `Sources/` (source), `Tests/` (tests), `Package.swift` (SwiftPM manifest). Xcode projects use `*.xcodeproj` / `*.xcworkspace`. Git-ignored: `.build/`.
