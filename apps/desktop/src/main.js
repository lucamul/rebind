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

function renderPages(inspection) {
  pagesEl.innerHTML = "";
  for (const page of inspection.pages) {
    const pageEl = document.createElement("section");
    pageEl.className = "page";

    const label = document.createElement("div");
    label.className = "page-label";
    label.textContent = `Page ${page.index + 1} of ${inspection.page_count}`;
    pageEl.appendChild(label);

    if (page.paragraphs.length === 0) {
      const empty = document.createElement("div");
      empty.className = "empty-page";
      empty.textContent = "(no extractable text — likely an image-only page)";
      pageEl.appendChild(empty);
    }

    for (const para of page.paragraphs) {
      const p = document.createElement("p");
      p.className = "paragraph";
      if (para.italic) p.classList.add("italic");
      if (para.bold) p.classList.add("bold");
      p.textContent = para.text;
      pageEl.appendChild(p);
    }

    pagesEl.appendChild(pageEl);
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
    renderPages(inspection);
  } catch (err) {
    status.textContent = `error: ${err}`;
  }
});
