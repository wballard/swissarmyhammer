---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m12mvyxd2kf0pvyhg1j7fyxp
  text: |-
    Picked up. Research findings, verified against HEAD (branch `kanban`), not against the card text:

    **Code is confirmed correct.** `tag_refs` is `aliased_list_param(op, "tags", "tag")` in `crates/swissarmyhammer-kanban/src/dispatch.rs`. `aliased_list_param` reads both keys through `list_param`, which routes through `ref_list`. So the singular `tag` takes an array, a scalar, and a stringified array — every shape `tags` takes. No behaviour change is needed, exactly as the card says.

    **The false claim is still present in three places, not two.** The card names two; a repo-wide search for "one-element" finds a third that repeats the same statement:
    1. `crates/swissarmyhammer-kanban/src/dispatch.rs` — the `tag_refs` doc comment: "The singular `tag` is accepted as a one-element alias, because that is the key `tag task` teaches."
    2. `crates/swissarmyhammer-tools/src/mcp/tools/kanban/description.md` — "The singular `tag` is accepted as a one-element alias."
    3. `crates/swissarmyhammer-kanban/src/dispatch/tests/tags.rs` — the doc comment on `dispatch_tag_task_accepts_the_plural_tags_key`: "`tags` is documented as the canonical key and `tag` as its one-element alias". This one is worse than stale prose: it cites the documentation as the authority for the narrowing, so leaving it keeps a pointer to the claim the other two fixes remove.

    **The model wording already exists** — `^n36mc1q` set it for `assignee`, and the two params must read consistently:
    - `description.md`: "The singular `assignee` is an alias for the same list and takes every shape `assignees` takes."
    - `dispatch.rs` on `explicit_assignee_refs`: "... so it accepts every shape the plural key does — the alias names the key, it does not narrow the shape."

    **Places checked and found clean — no edit needed:**
    - `doc/src/reference/kanban-cli.md` — holds no `--tags` or `--tag` documentation at all, and no alias claim. This also keeps the work clear of the open defect `^18kd3j9` (the CLI does not route `--tags` through `ref_list`), because nothing here describes CLI tag input.
    - `crates/swissarmyhammer-kanban/src/schema.rs` — carries no alias prose; its tag references are op registration and the `list tasks` filter-param coverage test.
    - `builtin/skills/` — the only "alias" matches are the coverage skill references talking about Rust/Python/TS type aliases. Unrelated.
    - `description.md` "Scoping a listing" — correctly states each atom param takes ONE value, which matches `scalar_filter_param`. `tag` there is a filter atom, NOT the list alias, so it must stay as it is. The two meanings of the key `tag` must not be merged when rewording.

    **Test coverage of the claim, measured:**
    - Proven under the singular `tag` key already: array on `tag task` (`dispatch_tag_task_array_applies_one_tag_per_element`), stringified array on `tag task` (`dispatch_tag_task_stringified_array_applies_one_tag_per_element`), array on `update task` (`dispatch_update_task_singular_tag_accepts_an_array_and_rejects_junk`), scalar on `add task` (`dispatch_add_task_singular_tag_applies`).
    - Gap: no fixture puts a stringified array under the singular `tag` on the `add task` / `update task` path. That path is the one the `tag_refs` doc comment describes, so the corrected comment would rest on reading the code there, which is what the card forbids. One new test closes it.
  timestamp: 2026-08-27T22:18:53.741242+00:00
- actor: claude-code
  id: 01m12my7x0hw8928k5qbfyfpm0
  text: |-
    Card is NOT already satisfied — checked with `git log -S` on the phrase, because two later commits touched `description.md` and could have closed it by accident:

    - `74d0cacc4` introduced "one-element alias" for both `assignee` and `tag`.
    - `24b5d687e` (the `^n36mc1q` fix) removed only the `assignee` half.
    - `7af361e16` edited `description.md` for the `tag task` / `untag task` routing and `f8fc8394c` edited it again for `list tasks` scoping. Neither touched the `tag` alias sentence.

    So the `tag` claim has survived every later edit of the file and is live at HEAD (`91fc23cfb`).

    The `24b5d687e` diff also fixes the shape of this card's work. It did three things, and this card must do the matching three:
    1. `description.md` — "The singular `assignee` is an alias for the same list and takes every shape `assignees` takes."
    2. `dispatch.rs` doc comment — "... an alias read through the same [`list_param`] path, so it accepts every shape the plural key does — the alias names the key, it does not narrow the shape."
    3. Added `dispatch_add_task_singular_assignee_array_shape_persists`, a fixture for the singular key's array shape. This is the precedent for the test this card asks for, so the new `tag` test follows it rather than inventing a form.
  timestamp: 2026-08-27T22:20:08.480920+00:00
position_column: doing
position_ordinal: '8380'
title: Singular tag key docs repeat the corrected one-element alias claim
---
`^n36mc1q` corrected two places that described the singular `assignee` key as "a one-element alias". The identical phrasing survives for the singular `tag` key:

- `crates/swissarmyhammer-kanban/src/dispatch.rs:250`
- `crates/swissarmyhammer-tools/src/mcp/tools/kanban/description.md:26`

Both blame to `74d0cacc48` (2026-07-30), not to the `^n36mc1q` commits, so the reviewer correctly ruled them out of that task's scope.

## This is docs only — the code is already right

`tag_refs` routes the singular `tag` key through `list_param`, exactly as `assignee` now does. So `tag: ["urgent"]` already works. The defect is that the docs describe a narrowing the code does not perform, which is the mirror image of the `^n36mc1q` finding: there the doc promised shape tolerance the code lacked, here the doc implies a restriction the code does not impose.

Do not "fix" this by changing `tag_refs`. Verify first that it calls `list_param` for both keys, then correct only the prose.

## Required change

Reword both places the way `^n36mc1q` reworded the `assignee` text: the singular names the key, it does not narrow the shape. Match that wording so the two params read consistently.

`description.md` is the tool's user-facing contract, so its wording matters more than the internal doc comment.

## Acceptance

- Neither place claims the singular `tag` key takes only one element.
- A test proves `tag: ["urgent"]` and a stringified array both work under the singular key — if no such test exists, add one, because the claim in the docs should rest on a fixture rather than on reading the code.
- No behavior change; `tag_refs` is untouched and every existing test passes unedited.

Found by the review of `24b5d687e` while closing ^n36mc1q. #bug #kanban