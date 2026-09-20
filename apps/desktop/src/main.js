// Minimal wiring: no bundler yet. `withGlobalTauri` in tauri.conf.json
// exposes `window.__TAURI__` directly. The file dialog is called as a
// raw plugin command (`plugin:dialog|open`) rather than through
// `@tauri-apps/plugin-dialog`'s JS wrapper, for the same reason — the
// wrapper is just a thin call to this anyway.

const status = document.getElementById("status");
const pathLabel = document.getElementById("path");
const pagesEl = document.getElementById("pages");
const button = document.getElementById("open");

const invoke = window.__TAURI__.core.invoke;

const GUESS_LABEL = {
  continues: "leans: continues",
  new_section: "leans: new section",
  unknown: "no clear lean",
};

function conflictMarker(conflict) {
  const el = document.createElement("div");
  el.className = "conflict";
  const pct = Math.round(conflict.confidence * 100);
  el.textContent = `⚠ page break uncertain here — ${GUESS_LABEL[conflict.guess]} (${pct}% confidence)`;
  return el;
}

function render(inspection) {
  pagesEl.innerHTML = "";

  const conflictsAfter = new Map(); // block id -> conflict, for the block right before the boundary
  for (const c of inspection.conflicts) {
    conflictsAfter.set(c.before_block, c);
  }

  const summary = document.createElement("div");
  summary.className = "summary";
  const conflictCount = inspection.conflicts.length;
  summary.textContent =
    conflictCount === 0
      ? `${inspection.page_count} pages, ${inspection.blocks.length} paragraphs recovered — no uncertain page breaks`
      : `${inspection.page_count} pages, ${inspection.blocks.length} paragraphs recovered — ${conflictCount} page break${conflictCount === 1 ? "" : "s"} need review`;
  pagesEl.appendChild(summary);

  for (const block of inspection.blocks) {
    const p = document.createElement("p");
    p.className = "paragraph";
    if (block.italic) p.classList.add("italic");
    if (block.bold) p.classList.add("bold");
    p.textContent = block.text;
    pagesEl.appendChild(p);

    const conflict = conflictsAfter.get(block.id);
    if (conflict) {
      pagesEl.appendChild(conflictMarker(conflict));
    }
  }
}

button.addEventListener("click", async () => {
  let selection;
  try {
    // The command's single struct parameter is named `options`, so
    // Tauri's arg binding requires it nested under that key.
    selection = await invoke("plugin:dialog|open", {
      options: {
        title: "Open PDF",
        multiple: false,
        directory: false,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      },
    });
  } catch (err) {
    status.textContent = `error opening file picker: ${err}`;
    return;
  }

  if (!selection) {
    return; // user cancelled
  }
  const path = typeof selection === "string" ? selection : selection.path;

  pathLabel.textContent = path;
  pagesEl.innerHTML = "";
  status.textContent = "loading…";
  try {
    const inspection = await invoke("open_pdf", { path });
    status.textContent = "";
    render(inspection);
  } catch (err) {
    status.textContent = `error: ${err}`;
  }
});
