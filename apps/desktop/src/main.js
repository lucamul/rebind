// Minimal wiring: no bundler yet. `withGlobalTauri` in tauri.conf.json
// exposes `window.__TAURI__` directly, which is enough while this is
// still a single static page. Revisit once the review UI needs a real
// framework (and a real file-picker dialog instead of a path field).

const status = document.getElementById("status");
const pathInput = document.getElementById("path");
const button = document.getElementById("open");

button.addEventListener("click", async () => {
  const path = pathInput.value.trim();
  if (!path) {
    status.textContent = "enter a path first";
    return;
  }
  status.textContent = "loading…";
  try {
    const summary = await window.__TAURI__.core.invoke("open_pdf", { path });
    status.textContent = JSON.stringify(summary, null, 2);
  } catch (err) {
    status.textContent = `error: ${err}`;
  }
});
