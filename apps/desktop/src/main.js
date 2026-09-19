// Minimal wiring: no bundler yet. `withGlobalTauri` in tauri.conf.json
// exposes `window.__TAURI__` directly, which is enough while this is
// still a single static page. The file dialog is called as a raw
// plugin command (`plugin:dialog|open`) rather than through
// `@tauri-apps/plugin-dialog`'s JS wrapper, for the same no-bundler
// reason — the wrapper is just a thin call to this anyway.

const status = document.getElementById("status");
const pathLabel = document.getElementById("path");
const button = document.getElementById("open");

const invoke = window.__TAURI__.core.invoke;

button.addEventListener("click", async () => {
  let selection;
  try {
    selection = await invoke("plugin:dialog|open", {
      title: "Open PDF",
      multiple: false,
      directory: false,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
  } catch (err) {
    status.textContent = `error opening file picker: ${err}`;
    return;
  }

  if (!selection) {
    return; // user cancelled
  }
  // The dialog plugin's FilePath can come back as a plain string or
  // (rarely, e.g. some Android content:// URIs) an object — handle
  // both rather than assume.
  const path = typeof selection === "string" ? selection : selection.path;

  pathLabel.textContent = path;
  status.textContent = "loading…";
  try {
    const summary = await invoke("open_pdf", { path });
    status.textContent = JSON.stringify(summary, null, 2);
  } catch (err) {
    status.textContent = `error: ${err}`;
  }
});
