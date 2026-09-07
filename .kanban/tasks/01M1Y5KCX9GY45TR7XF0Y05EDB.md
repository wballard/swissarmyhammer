---
assignees:
- claude-code
depends_on:
- 01M1Y5GD85944ACE2XS6SA5XWJ
position_column: todo
position_ordinal: fff980
title: Stop installing swiftformat in CI
---
## What

`.github/workflows/ci.yml` installs `swiftformat` with Homebrew on the self-hosted macOS runner. No rule uses that tool after the task "Move idioms-swift to the toolchain swift format". Remove the install.

This task depends only on "Move idioms-swift to the toolchain swift format", because that is the last rule that runs swiftformat. `missing-docs-swift` runs swiftlint, and CI keeps installing swiftlint.

Files to change:
- `.github/workflows/ci.yml`

Remove only this block:

```
if ! command -v swiftformat &>/dev/null; then
  brew install swiftformat
fi
```

KEEP the `swiftlint` install, because `disallowed-constructs-swift`, `function-length-swift` and `magic-numbers-swift` still run it. KEEP the `periphery` install, with its `brew uninstall --cask` line, because `dead-code-swift` still runs it.

The comment above the install block reads "`swiftlint`, `periphery` and `dart` ship only through Homebrew". It names no swiftformat, thus it is already correct. Change it only if you find it wrong.

The `swift --version` step stays, and it becomes more important: the toolchain is now the formatter. Add a guard that makes the workflow fail with a clear message when the runner's Swift version is below the floor the measurement task states. Row 8 of that task holds the floor and the runner's own version.

Name a step by its name and a block by its text. Do not name a line number, because a line number moves.

## Acceptance Criteria
- [ ] `.github/workflows/ci.yml` names `swiftformat` nowhere.
- [ ] The workflow still installs `swiftlint` and `periphery`.
- [ ] The workflow fails with a clear message when the runner's Swift version is below the stated floor.
- [ ] CI passes on the self-hosted macOS runner, and every Swift tool rule reports as healthy.

## Tests
- [ ] Run `rg swiftformat .github/`. The command finds nothing.
- [ ] Run the full Swift test set on the runner: `cargo nextest run -p swissarmyhammer-validators swift`. Every test passes with no `swiftformat` on the PATH.
- [ ] Run `sah doctor` on the runner. Every Swift tool rule reports as healthy.
- [ ] Test the version guard: run the guard with a version string below the floor, and it fails with the stated message.
- [ ] Push the branch and read the CI result. The macOS job is green.

## Workflow
- Use `/tdd` — write the failing tests first, then write the code that makes them pass. #swift