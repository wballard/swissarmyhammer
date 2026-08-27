---
assignees:
- claude-code
position_column: todo
position_ordinal: fff780
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