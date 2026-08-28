---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m14a01x2zgemmtwt1r4yf44s
  text: |-
    ### Research — current shapes verified

    Picked up; card moved to `doing`. The `move task` that moved it returned `"_plan": {..., "entries": []}` — the bug reproduces live.

    **1. `ListTasks::execute` return shape (crates/swissarmyhammer-kanban/src/task/list.rs).** The card quoted `{ "tasks": [...], "count": N }`. The CURRENT shape is wider — pagination landed after the card was written:

    ```json
    { "tasks": [...], "count": <items on this page>, "total": <items after filtering>,
      "page": N, "page_size": N, "total_pages": N }
    ```

    The key IS `tasks`. Card item 1 stands.

    **2. Two defaults make a one-line `tasks["tasks"]` fix incomplete.**

    - `DEFAULT_PAGE_SIZE = 10`. `ListTasks::new()` returns only the first TEN cards. On this board (hundreds of cards) the affected card is almost never in that page, so the card's acceptance — "the affected task appears in `entries[]._meta.id`" — would hold only on a board under ten cards. Every existing `*_plan_carries_affected_task_id` test seeds one card, so a first-page-only fix passes the tests and stays broken in production.
    - `excludes_done()` is `exclude_done.unwrap_or(column.is_none())`, so an unscoped `ListTasks::new()` DROPS the done column. Acceptance criterion 2 ("done to completed") would stay unreachable, and `complete task` — which leaves its card on the board — would lose that card from the plan.

    So the plan listing must set `exclude_done` false and page to completeness. `MAX_PAGE_SIZE = 100` clamps `page_size`, so one call cannot cover a big board; the builder follows `total_pages`. This also satisfies the ACP rule the module header quotes: "Complete plan lists must be resent with each update".

    `MAX_PAGE_SIZE` is `pub` in the private `task::list` module and is NOT re-exported, so `task/mod.rs` needs `pub use list::{ListTasks, MAX_PAGE_SIZE};`.

    **3. Card item 3 — `task_to_plan_entry` against a REAL element. CHECKED.** `task_entity_to_json` builds `"position": {"column": ..., "ordinal": ...}`, and `SLIM_TASK_FIELDS` keeps `position`. So `task["position"]["column"]` IS correct on a real `list tasks` element, under both detail levels. `task["id"]` and `task["title"]` are correct too.

    One real gap found: `parse_detail(None)` is `TaskDetail::Slim`, and the slim allowlist deliberately drops `description`. So `task_to_plan_entry`'s `with_notes(desc)` never fires and every entry's `_meta.notes` is null. Keeping slim is correct here — pulling every card's description into every mutation response is exactly the prompt-token blowup the pagination comment in `list.rs` warns about — but the null is a consequence of the detail level, not an accident, and is documented as such.

    **4. Requirement 2 (no silent degrade).** A missing or non-array `tasks` key warns through `tracing::warn!` and returns `None`, matching the existing `Err` arm. No `_plan` beats an empty `_plan`.
  timestamp: 2026-08-28T13:47:22.402643+00:00
- actor: claude-code
  id: 01m14b4mzzjnk3kkbd6hvc2d00
  text: |-
    ### implement — changed

    **RED first, both waves.** `test_plan_entries_name_every_card_with_its_column_status` and `test_plan_entries_reach_past_one_list_tasks_page` were written and run BEFORE any production edit. They failed on the defect itself:

    ```
    `complete task` left card 01M14A667H4XESRJG5AZXFP2YT out of `_plan.entries`,
    got: {..."_plan":{"_meta":{...},"entries":[]},...}

    card 01M14A667HB546R0RSHDRG5XFH is on the board but missing from
    `_plan.entries` (0 of 101 cards planned)
    ```

    Second wave — `test_read_task_page_refuses_a_listing_it_cannot_read` and `test_read_task_page_reads_a_well_formed_listing` — failed to compile against the absent symbol, then went green with the guard.

    **What changed.**

    `crates/swissarmyhammer-tools/src/mcp/tools/kanban/mod.rs`
    - `read_task_page(&Value) -> Option<(&[Value], u64)>` reads one page's cards and its page count out of the response OBJECT, under the named keys `LIST_TASKS_ARRAY_KEY` ("tasks") and `LIST_TASKS_TOTAL_PAGES_KEY` ("total_pages"). A shape it cannot read yields `None` — never an empty list.
    - `list_all_tasks_for_plan(ctx)` walks every page: `with_exclude_done(false)`, `with_page_size(MAX_PAGE_SIZE)`, following `total_pages`. Both defaults had to be overridden — see the research note above. It `tracing::warn!`s and returns `None` on a failed list AND on a shape mismatch, so `build_plan_data` attaches no `_plan` rather than an empty one.
    - `build_plan_data` now consumes that list. The entry mapping and `task_to_plan_entry` are untouched — they were correct, only starved.

    `crates/swissarmyhammer-kanban/src/task/mod.rs` — `pub use list::{ListTasks, MAX_PAGE_SIZE};`. `MAX_PAGE_SIZE` was `pub` inside the private `task::list` module, so no caller outside the crate could ask for the widest page.

    **Blast radius — one real break, found and fixed.** `swissarmyhammer-cli::kanban_cli_tests::test_kanban_task_update` asserted `!result.stdout.contains("Updated Title")` over the WHOLE response, to prove the thin ack echoes no fields. The plan carries every card's title as `entries[].content` by contract, so the assertion was measuring the plan, not the ack. It now parses the YAML, drops `_plan`, and asserts on the ack alone — plus a NEW assertion that the full response DOES carry the title, so the split cannot silently gut the test. `ack_without_plan` also asserts a `_plan` was actually present to remove.

    Checked the other `ListTasks` consumers for the same misread: `dispatch.rs` and `apps/kanban-app/src/cli.rs` both forward the whole object and never call `as_array()` on it. No sibling defect.

    **Verification.**
    - `cargo nextest run --workspace` — 14259 passed, 0 failed, 0 skipped.
    - `cargo clippy --workspace --all-targets -- -D warnings` — clean.
    - `cargo fmt` — applied.
    - The twelve `*_plan_carries_affected_task_id` tests: all green.

    **Not yet observable in this session.** The MCP server answering these `kanban` calls is a binary built before this change, so `_plan.entries` in this session's tool results stays `[]`. The behavior is proven by the tests above, not by a live call.

    - evidence: 3 files — crates/swissarmyhammer-tools/src/mcp/tools/kanban/mod.rs, crates/swissarmyhammer-kanban/src/task/mod.rs, apps/swissarmyhammer-cli/tests/kanban_cli_tests.rs; `cargo nextest run --workspace` 14259 passed / 0 failed
    - next: /review
  timestamp: 2026-08-28T14:07:21.599500+00:00
position_column: doing
position_ordinal: '8380'
title: _plan.entries is always empty — build_plan_data reads an object as an array
---
`build_plan_data` in `crates/swissarmyhammer-tools/src/mcp/tools/kanban/mod.rs` never produces a single plan entry. Every `_plan` the kanban MCP tool attaches carries `"entries": []`.

## Cause

`build_plan_data` calls `ListTasks::new().execute(ctx)` and then `tasks.as_array()`.

`ListTasks::execute` (`crates/swissarmyhammer-kanban/src/task/list.rs`) returns an OBJECT, not an array:

```json
{ "tasks": [ ... ], "count": 12 }
```

So `as_array()` gives `None`, `unwrap_or(&Vec::new())` supplies an empty list, and the `.map()` that builds the entries never runs. The failure is silent — there is no error and no warning.

## Evidence

A live `move task` call against this board, which holds hundreds of cards:

```json
"_plan": {
  "_meta": { "affected_task_id": "01KYSFNAHGT9827596R1T92GNJ",
             "source": "swissarmyhammer_kanban",
             "trigger": "move task" },
  "entries": []
}
```

`_meta.affected_task_id` is correct. Only `entries` is dead.

## Why this matters

The whole purpose of `_plan` is the entries list. The module header quotes the ACP rule "Complete plan lists must be resent with each update". The tool resends an empty list every time, so an ACP agent that emits Plan notifications from `_plan` shows the user an empty plan. `task_to_plan_entry` and the `PlanEntryStatus` / `PlanEntryPriority` mapping beside it are dead code today.

## Required change

1. Read the array out of the object: `tasks["tasks"].as_array()`. Verify against `ListTasks::execute` rather than assuming the key.
2. A shape mismatch must not stay silent. An unreadable task list should warn or error, the same way the `Err` arm already does, instead of degrading to an empty plan.
3. Check `task_to_plan_entry` against a REAL `list tasks` element. It reads `task["position"]["column"]`; confirm that the enriched shape `list tasks` returns actually carries that path, because nothing has ever exercised it.

## Acceptance

- A read-back test asserts `_plan.entries` names the cards on the board — at minimum, that the affected task appears in `entries[]._meta.id` for an operation that leaves the card on the board. The test must fail before the change.
- Entry `status` maps from the column: done to completed, doing to in_progress, else pending.
- The twelve `*_plan_carries_affected_task_id` tests in `mcp::tools::kanban::tests` stay green.

Found while adding the `affected_task_id` read-back tests for ^1t92gnj. Deliberately NOT fixed there: that card is test-only and on its final review round, and this is a production behavior change to every `_plan` consumer. #bug #kanban