// Minimal wiring: no bundler yet. `withGlobalTauri` in tauri.conf.json
// exposes `window.__TAURI__` directly, which is enough while this is
// still a single static page. Revisit once the review UI needs a real
// framework.

const status = document.getElementById("status");
const button = document.getElementById("ping");

button.addEventListener("click", async () => {
  try {
    const reply = await window.__TAURI__.core.invoke("ping");
    status.textContent = reply;
  } catch (err) {
    status.textContent = `error: ${err}`;
  }
});
