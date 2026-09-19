# Rebind

A desktop app that turns a fixed-layout PDF into a clean, structured
manuscript — the same shape as a real source project (headings,
paragraphs, verse), not a lossy direct-to-EPUB conversion.

## Why

PDF has already thrown away the structure a reflow needs: it knows
glyphs at (x, y) positions, not "this is a paragraph" or "this page
break is deliberate." Reconstructing that is inference, and pure
heuristics get it wrong in predictable places — a chapter break versus
a paragraph that just happened to run out of page, a running header
versus real body text, a hyphen from justification versus a genuinely
hyphenated word.

Rebind's answer: get the obvious cases right automatically, and
surface only the genuine ambiguity — as a **conflict** — for a human
to resolve in a few seconds. The output is a real structured document,
not a black box; from there, EPUB, print, or anything else compiles
cheaply and correctly every time, instead of re-running the hard
reconstruction whenever you want a different target.

## Layout

```
crates/rebind-core/    UI-agnostic structure-recovery engine.
                        PDF bytes in, a Document + a list of
                        Conflicts out. No windows, no rendering.

apps/desktop/           The Tauri app: renders conflicts, takes the
                         user's resolutions, applies them back.
```

## Status

Early scaffold. `rebind-core` has the core `Document`/`Conflict` data
model and nothing else yet — no PDF ingestion. The desktop app is a
bare Tauri window proving the wiring works end to end.

## Developing

```
cargo build                          # whole workspace
cargo test -p rebind-core            # core crate's tests
cd apps/desktop && cargo tauri dev   # the app, with a live window
```
