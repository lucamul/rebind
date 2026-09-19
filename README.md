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

Early scaffold, PDF ingestion wired but not yet structure recovery.

`rebind-core` can bind to PDFium and extract every page's text as
positioned, font-tagged characters (`RawChar`/`RawPage`) — the raw
material the recovery pass (page-break classification, header/footer
detection, paragraph reconstruction) will turn into a `Document` full
of `Conflict`s. That pass doesn't exist yet.

The desktop app can open a PDF by path and show a basic summary (page
count, first-page size and char count) — proof the whole stack works
end to end, not a real review UI yet.

## PDFium

Ingestion uses [PDFium](https://pdfium.googlesource.com/pdfium/) (BSD-
3-Clause) via [pdfium-render](https://github.com/ajrcarey/pdfium-render),
for real text-layer extraction with position and font data, and later,
page rendering for the review UI. The prebuilt dynamic library isn't
committed (~7MB, platform-specific) — fetch it once after cloning:

```
scripts/fetch-pdfium.sh
```

## Developing

```
cargo build                          # whole workspace
cargo test -p rebind-core            # core crate's tests (incl. a
                                      # real PDF fixture, no mocks)
cd apps/desktop && cargo tauri dev   # the app, with a live window
```
