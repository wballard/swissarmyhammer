---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m14dk3jfm9zqk3bg4p4w6rsd
  text: |
    The card is not damaged. There is nothing to recover. I prove it three ways.

    ## 1. The changelog replays to the file on disk, byte for byte

    I replayed all 17 records of `.kanban/tasks/01M11YRV7M56YWPCJNG7RH0BVJ.jsonl`
    from an EMPTY document. Each `forward_patch` is a unified diff. I applied them
    in order with the `patch` utility. All 17 applied clean.

    The result is byte-identical to the file on disk:
    `sha256 351db22cbc9790378a21a2dc324b68361467694e9ca3e363f720a2deb1eb9ffd`,
    16725 bytes, both. `diff` reports no difference.

    So the file holds exactly what its history says it must hold. No title was
    lost. No body text was lost.

    ## 2. The title is present, and it never changed

    The front matter of the card ends at line 252, the first WHOLE line equal to
    `---` after the opening one. Line 251, inside that front matter, reads:

        title: kanban task write corrupts a card whose front matter holds a `---` run

    That is the same title the CREATE record wrote. All four committed versions of
    the file carry it: `c157c8a72`, `64c3e758f`, `c743d620b`, `7af361e16`.

    ## 3. The `title: Untitled` line is the card's own prose

    The match is line 267. It is in the BODY, after the closing delimiter, inside a
    fenced code block. The block quotes the corrupt output the card reports:

        ```
            | the refusing path | status | stdout | stderr |
            |
        title: Untitled
        ---
        |---|---|---|
        ```

    The CREATE record already carried that line, at its own line 25. The card was
    BORN with the string. The defect never touched it.

    ## Where the false report came from

    `grep "^title: Untitled$"` cannot tell front matter from body prose. The sweep
    verification on ^4nzhg4s used that grep, counted 1 match, and read the count as
    damage. The two implement comments on ^7rh0bvj had already reported the truth
    twice: "this card itself, whose description quotes the string as prose".

    ## The comparison the card asks for

    The card asks me to compare my reconstruction against the text quoted in the
    comments on ^4nzhg4s and on ^7rh0bvj.

    - AGREE, on the title. My reconstruction gives the title this card names, word
      for word.
    - AGREE, on the body. My reconstruction holds the corrupt-output block, the
      "What happened", "The blast radius", "Work" and "Done when" sections, and the
      two checked review findings.
    - DIFFER, on the conclusion. The ^4nzhg4s comment says ^7rh0bvj "is already
      `Untitled` in HEAD, damaged by the very defect it documents". That is wrong.
      The comments on ^4nzhg4s quote no body text of ^7rh0bvj, so they gave nothing
      to compare against. The `.jsonl` replay is the only full source, and it
      matches the disk.

    ## Board sweep

    I wrote a front-matter-aware sweep. It splits each card at the first whole-line
    `---`, then reads the `title` key of the front matter only.

    - Before: 3314 cards scanned, 2 suspect.
    - After: 3314 cards scanned, 2 suspect. I made no write, so the count cannot
      move.

    The 2 are the trashed tasks the card names as genuinely empty:
    `.trash/01KPE175H9DPH0VAYR02JRV2MF.01KPE17AZSZ1PMJVTPAWM7A9J9.md` and
    `.trash/01KQQ51B1XQ6YDA7FN6JJF0DZ3.01KQQ51B32BPBZBGFX233P8CJD.md`. Each is 69
    bytes and holds no body. Their changelogs show `title: Untitled` and no other
    title, so neither ever had one.

    Zero LIVE cards carry `title: Untitled` in their front matter. The card's "Done
    when" is met.

    ## Naive grep counts, for the record

    - `grep -rl "^title: Untitled$" .kanban/tasks/`: 3 before, 3 after. Two are the
      trashed tasks; one is this prose match.
    - `grep -l "^title: Untitled$" .kanban/tasks/*.md`: 1 before, 1 after. That one
      is the prose match, not damage.

    ## No source change

    I changed no Rust file. `git status` reports no change outside `.kanban/`.
    `cargo fmt --all -- --check` is clean. `cargo clippy --workspace --all-targets
    -- -D warnings` exits 0.

    ## For the next agent

    Do not file this card again. Use a front-matter-aware check, never a bare grep,
    to measure `Untitled` on this board. The write-side guard from ^7rh0bvj
    (`join_frontmatter_body`) now refuses front matter that holds a `---` line, so
    a card cannot break this way again.
  timestamp: 2026-08-28T14:50:12.431507+00:00
- actor: claude-code
  id: 01m14jz2jrr1g0z0ea1af2ksdy
  text: |-
    ### finish iteration 1 — no-change
    - implement: no-change — there was nothing to recover. No source file and no board data were written.
    - test: not run — no code changed. fmt and clippy were confirmed clean anyway.
    - commit: board comments only
    - review: not applicable — no code changed, so a review pass would scan nothing and could not be read as a pass
    - next: none — the card is closed as a false alarm

    The card's premise was wrong, and the error was mine. I wrote the sweep-verification comment on ^4nzhg4s that said this card was "already `Untitled` in HEAD, damaged by the very defect it documents", and I filed this card on the strength of it. I had measured with `grep "^title: Untitled$"`, which cannot tell front matter from body prose.

    The single match is line 267 of `01M11YRV7M56YWPCJNG7RH0BVJ.md` — in the BODY, inside a fenced code block that quotes the corrupt output the card reports. The card's CREATE record already carried that line, so the card was born with the string and the defect never touched it. Line 251, in the front matter, holds the real title: `kanban task write corrupts a card whose front matter holds a ` + backtick-`---`-backtick + ` run`.

    Proof, three ways, two by the implementer and one by me independently:
    - All 17 records of the card's `.jsonl` replayed from an empty document with `patch` produce a file byte-identical to disk: sha256 351db22cb…, 16725 bytes.
    - All four committed versions of the file carry the same title.
    - A front-matter-aware scan of 3067 live cards finds ZERO with a missing or `Untitled` title. The only two `Untitled` files are empty trashed tasks whose changelogs show they never had a title.

    The card's "Done when" — no card that ever had a title reads `Untitled` — is therefore already met, and was met before this card was filed.

    Standing correction for anyone measuring this again: never use a bare grep for `title:` on this board. Split the file at the first WHOLE line equal to `---` and read the front matter only. The write-side guard from ^7rh0bvj (`join_frontmatter_body`) now refuses front matter holding a `---` line, so a card cannot break this way again.
  timestamp: 2026-08-28T16:24:07.512465+00:00
position_column: done
position_ordinal: ffffffffffffffffffffffffffffffffffffffffffb980
title: Recover the title and body of card 01M11YRV7M56YWPCJNG7RH0BVJ from its changelog
---
`.kanban/tasks/01M11YRV7M56YWPCJNG7RH0BVJ.md` reads `title: Untitled`. The card is ^7rh0bvj, "kanban task write corrupts a card whose front matter holds a `---` run". It lost its own title and part of its body to the very defect it documents.

## What is known

The sweep on ^4nzhg4s measured the board before and after its own work. The `title: Untitled` count was 1 both times, and this card was that one. So the damage predates the sweep and the sweep neither caused nor cured it.

The defect that ate it is fixed. ^7rh0bvj is in `done`: `join_frontmatter_body` now refuses to write front matter holding a whole line equal to `---`, so no further card can break this way. Only the recovery is left.

## Work

- Read `.kanban/tasks/01M11YRV7M56YWPCJNG7RH0BVJ.jsonl`. The changelog holds the create record and every patch, so the original title and body can be rebuilt from it.
- Restore the title and the lost body text.
- Compare the rebuilt card against the version quoted in this session's comments on ^4nzhg4s and ^7rh0bvj, which reproduce much of the original text.
- Sweep for any other card whose title reads `Untitled` but whose changelog shows a real title. Two trashed tasks are genuinely empty and are not damage.

## Done when

The card carries its real title and its full body, and `grep -c "^title: Untitled$" .kanban/tasks/*.md` answers zero for cards that ever had a title. #kanban #bug