# Dual-degree overlap: why it is slow, and what changing it would do

This note records how multi-degree `/generate_schedule` spends its time, what three targeted changes in `overlap_planner.rs` would do, how that shows up in the app, and why those changes have not been made yet.

Measured on Fly (`POST /generate_schedule`, `latency_ms` in Neon, 2–13 Aug 2026):

- No overlap path (one program, or two programs that never enter this search): **~13 ms** average
- Overlap path: **~15 s** average, often **25–35 s** (EE + Wharton, EE + MS)
- Local release, CIS only: **~0.4 ms**

Semester packing is not the slow part. The extra cost starts when a second program turns on `compute_overlap_plan`.

---

## What happens today

A generate with two or more programs always runs overlap discovery, including after a CU-limit or gap toggle. Single-program generates skip it (`overlap_plan_applicable` requires 2+ degrees).

```
Validate each degree (which slots are still open?)
  → extract open slots, compile a CourseMatcher each
  → keep overlap-eligible slots
  → build a candidate course set per slot
  → invert: course → slots it can fill
  → for slots with no set: test every peer candidate
  → every course that hit 2 degrees becomes slot pairs
  → pack the semester grid
```

### Open slots and matchers

After each tree is audited, leftover requirements become matchers:

- a short named list (`CIS 4190` / `CIS 4210`)
- an attribute restriction (`any Humanities and Social Science course`)
- a department restriction
- a catch-all

Wharton unrestricted electives are catch-alls: `restriction(1)` with **no department, no attribute, no `no_school`**. There are five of them (`wh_unrestricted_electives(5)`). SEAS general/professional **pool flex** slots (`29:p0`, `29:p1`, …) compile to `CourseMatcher::Unrestricted` (“any valid course”).

### Eligibility is where the search goes wide

A slot is pulled into overlap if:

- its id contains `:p` or `:c` (pool flex / coverage) — **always**, even when the matcher is Unrestricted
- or it is a Restriction with empty department — the comment in code is explicit: *“Attribute, `no_school`, or fully unconstrained (e.g. WH Unrestricted Electives)”*

Dept-limited restrictions (e.g. “any CIS 1xxx”) are actually **excluded**. The planner searches the loosest slots, not the tightest.

### Candidate sets

- Named list → tiny set (the list).
- Attribute → attribute index (hundreds of courses, reasonable).
- Unconstrained / `no_school` Restriction → clone **every undergrad course in `courses.json`**, run `course_matches_restriction` (and mutex/equiv checks) on each, sort, **keep 800** (`MAX_CANDIDATES_PER_SLOT`).

Five identical Wharton unrestricted slots means five full-catalog scans, then five 800-course sets that are basically “the alphabetically first 800 undergrad codes,” not the best overlaps.

Pool flex slots cannot enumerate (`Unrestricted` → `candidates_for_matcher` returns `None`). They are handled next.

### Invert (loop 1)

For every slot that *did* get a set, every remaining course is run through `course_satisfies_matcher` **again** — the same predicate that just built the set — and recorded as `course → [slot ids]`.

### Peer probe (loop 2)

For every eligible slot with **no** set (the `:p` Unrestricted flex slots):

- for every *other* slot that has a set
- for every course in that set (up to 800)
- ask “does this flex slot accept it?”

For Unrestricted, the answer is always yes (`is_valid_course_code`). So every one of those 800 codes is attached to every SEAS flex slot.

### Pair explosion

Overlap only *keeps* pairs that span two degrees, but it still *builds* them. After the peer probe, a typical course looks like: “hits all 5 Wharton unrestricted slots + all SEAS flex slots.” The code trims to one flex per degree and still emits **5 (WH unrestricted) × 1 (SEAS flex)** pairs **per course**, times ~800 courses.

Then each pair is scored, explained, stuffed into hover hints, and mostly thrown away because “ACCT 1010 counts as Unrestricted Elective + General Elective” is not a real planning insight. The work to discover thousands of those pairs is still paid.

That is why **EE + Wharton is ~25 s** and **CAS BIOL + Wharton is ~3.4 s**: same algorithm, but SEAS has more pool/flex catch-alls, so the 800-wide sets get multiplied more.

On Fly this also runs **synchronously** on **1 shared CPU**, so one dual generate stalls other API calls until it returns.

---

## The three changes

### 1. Stop searching from unconstrained electives

**Today:** “WH Unrestricted Elective” and “SEAS General Elective flex” are treated as *places to look for shared courses*.

**They should not be search seeds.** A useful overlap is a **constrained** coincidence: `ESE 3010` is on EE’s probability list *and* Wharton’s STAT list; a writing seminar satisfies both WRIT rules; an attribute-H course satisfies both SSH-style requirements. A slot that accepts the whole catalog does not *discover* a coincidence. It only says “whatever you already found, I can eat it too.”

Eligibility should require a **narrow** matcher:

- keep: finite `OneOf` lists, WRIT, real attributes, and (optionally) typed `no_school` such as “non-Wharton”
- drop: Restriction with no dept/attr/`no_school`; `Unrestricted` even if the id is `:p`

Wharton unrestricted can still **receive** a course that was found via a real pair. Those slots just stop being 800-wide search keys.

**Why it is fast:** skip the full-catalog clone+filter per unrestricted slot, never insert 800 junk courses into `course_to_slots`, never generate 5×flex pairs for each of them.

### 2. Remove (or tightly bound) the nested peer loop

The file claims inverted-index discovery is “O(sum of candidate sizes), not O(slots²).” The peer loop reintroduces a product: catch-all flex slots have `None` instead of a set, so the code asks “which of the other slot’s 800 courses does this flex accept?” Answer: **all of them**.

That does not find new overlaps. It **broadcasts** every broad candidate onto every flex slot, which feeds the pair explosion above.

**After:** only invert courses that already sit in **two degrees’ enumerated sets**. If EE slot A and WH slot B both contain `ESE 3010`, that is an overlap. Flex electives no longer get a copy of every other slot’s 800 courses.

If “this specific overlap course can also sit in a general elective” is still wanted, that can be a **post-pass on the dozen suggested courses**, not a search over 800 catalog rows.

Matcher calls in that loop are individually cheap. The **downstream** HashMap of 800 courses × many slots, then `cross_degree_slot_pairs` + scoring + hint maps, is not.

### 3. Trust the candidate set (do not re-test it)

After `candidates_for_matcher` returns a set, the invert loop calls `course_satisfies_matcher` on every member with the **same** matcher. For a 5-course list that is noise. For an 800-course restriction set it is 800 extra `course_matches_restriction` calls per slot.

**After:** if the course is in `slot_candidates[i]`, push slot `i` and move on.

Smaller than (1) and (2), but it is pure duplicate work on the largest sets you still keep (attribute / `no_school` restrictions). It also removes a footgun: the two functions can drift.

### How they stack

| Step | Today (EE + WH, empty plan) | After 1–3 |
|---|---|---|
| Unconstrained WH electives | 5 × full-catalog scan → 800 codes each | Not search seeds |
| SEAS flex `:p` | No set; absorb every peer’s 800 codes | Not search seeds |
| Invert | Re-test every candidate | Membership in the set is enough |
| Pairs | ~800 courses × (unrestricted × flex) junk pairs | Only real two-list / attribute intersections |
| Grid packing | Same greedy packer, milliseconds | Unchanged algorithm |

---

## Effect on the app (functionality and outputs)

Overlap is not only a search. Its results are **rendered and packed**.

### What users see today from unconstrained pairs

When a pair has **no** single “fixed” suggested course, the scheduler turns it into an **overlap schedule group** (`req:overlap:…`): one 1.0 CU block on the grid labeled like `Unrestricted Electives (WH_NOFL) + General Electives (EE)`. The two individual `req:` placeholders are **suppressed**, so the packer places **one** CU instead of two.

Requirements-panel hover hints (`hints_by_slot`) also list up to 12 suggested codes on those rows. For unconstrained slots those codes are the alphabetically first catalog matches, not a curated dual-degree list.

When a pair **does** have a fixed course (typical for named-list / attribute overlaps), that course is added to the suggested schedule and claimed on both degrees.

### What would change after 1–3

**Unchanged (the useful part):**

- Named-list overlaps (EE STAT-style core ∩ Wharton STAT concentration, cross-listed CIS, etc.)
- Attribute overlaps (SSH / humanities-style restrictions)
- WRIT overlaps
- Hover text and paired blocks for those constrained pairs
- Per-degree requirement trees, taken/frozen behavior, CU limits, gap semesters

**Changed (the catch-all part):**

- Unrestricted elective rows and pool-flex rows would **stop** getting “also counts toward the other degree” hints sourced from the 800-wide catalog prefix.
- The grid would **stop** collapsing leftover WH unrestricted CU with leftover SEAS flex CU into a single overlap block, **unless** that pairing is reintroduced as a cheap structural rule (see tradeoff).
- Dual-degree plans could show **more open 1.0 CU placeholders** and therefore a **heavier** packed schedule (more cards, possibly an extra term) for students whose only remaining sharing was “any leftover elective on both sides.”

Excel/JPEG export follow the same grid, so those overlap blocks would disappear there too.

Change 3 has **no** product effect. It only skips a redundant predicate.

---

## Tradeoff

There **is** a real product tradeoff, and it is why unconstrained electives were made eligible on purpose.

At Penn, one future course **can** count as a leftover Wharton unrestricted elective **and** a leftover SEAS general/professional elective. The current search implements that by catalog-matching catch-all slots. The overlap **group** on the grid is the CU-saving representation of that rule: two requirement holes, one CU.

If you only stop using those slots as search seeds and do nothing else:

- Generates get much faster (the goal).
- Dual plans can look **harder** than they are: two 1 CU holes instead of one shared block.
- Students lose a (noisy) hint list on elective rows.

That CU-saving behavior does **not** require scanning 800 catalog courses. A cheaper substitute, if we still want the dual-count on leftover electives:

- After constrained overlaps are found, if both degrees still have leftover unrestricted/flex CU, emit **one structural pair per leftover CU** with no course list (or with the already-chosen overlap suggestions). Same grid collapse, no catalog product.

Until that substitute exists, (1)+(2) trade **latency** against **optimistic dual-elective CU packing**.

Change 3 has no such tradeoff.

---

## Why this has not been done yet

Not because it is unknown that dual generates are heavy. Because the current shape was a **deliberate feature + a cap that was supposed to be enough**:

1. **Product intent.** The eligibility comment names Wharton unrestricted electives on purpose. Dual-counting leftover electives is part of how dual-degree plans are supposed to look shorter than “major A CU + major B CU.”
2. **The 800 cap was the performance plan.** `MAX_CANDIDATES_PER_SLOT` and the inverted index were meant to keep this interactive. Truncation still leaves five 800-wide sets that the peer loop multiplies across every flex slot, so the cap does not bound the product.
3. **Fear of changing plan shape.** Overlap groups are tested and shown on the grid. Tightening eligibility without a structural elective-pair substitute would make some SEAS+Wharton (and similar) schedules look worse. That is a behavior change, not a pure speedup.
4. **It was not measured as 15–35 s until the latency work.** Neon `latency_ms` plus a local release bench showed single-degree milliseconds vs overlap-path tens of seconds. Before that, “dual is slower” was easy to attribute to packing or the small Fly VM.
5. **This note is explanation, not an implemented patch.** The three code changes are specified; they have not been applied in this work.

Lowering the 800 cap further, or wrapping `generate_schedule` in `tokio::spawn_blocking`, are extras. The cap only matters if catch-alls stay eligible. `spawn_blocking` keeps other requests alive during a slow generate; it does not shrink that generate.

---

## Related code

- [`degree_planner/src/overlap_planner.rs`](../../degree_planner/src/overlap_planner.rs) — eligibility, candidate sets, invert, peer loop, pairs
- [`degree_planner/src/scheduler.rs`](../../degree_planner/src/scheduler.rs) — `overlap_schedule_groups`, suppressing paired `req:` ids, claiming a fixed overlap course
- [`docs/backend/features.md`](./features.md) §8 — overlap discovery design
- Neon `schedule_generates.latency_ms` — production handler time (solver only, not TLS)
