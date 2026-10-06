/**
 * Push Debug Table — Phase 1b shell.
 * Loads wasm from ./pkg after: wasm-pack build push_wasm --target web --out-dir ../viewer/pkg
 */

const logEl = document.getElementById("action-log");
const statusEl = document.getElementById("wasm-status");
const btnLoad = document.getElementById("btn-load");
const btnPlay = document.getElementById("btn-play");
const btnStep = document.getElementById("btn-step");

function log(line) {
  const stamp = new Date().toISOString().slice(11, 19);
  logEl.textContent += `[${stamp}] ${line}\n`;
  logEl.scrollTop = logEl.scrollHeight;
}

let wasm = null;

async function loadWasm() {
  statusEl.textContent = "Loading WASM…";
  try {
    const mod = await import("./pkg/push_wasm.js");
    await mod.default();
    wasm = mod;
    const name = typeof mod.engine_name === "function" ? mod.engine_name() : "(no export)";
    statusEl.textContent = `WASM ready — ${name}`;
    btnPlay.disabled = false;
    btnStep.disabled = false;
    log(`Loaded engine: ${name}`);
  } catch (err) {
    statusEl.textContent = "WASM missing — build push_wasm first";
    log(`Load failed: ${err.message || err}`);
    log("Run: wasm-pack build push_wasm --target web --out-dir ../viewer/pkg");
  }
}

btnLoad.addEventListener("click", () => {
  loadWasm();
});

btnPlay.addEventListener("click", () => {
  log("Play bots — not wired until Phase 1b game loop exists.");
});

btnStep.addEventListener("click", () => {
  log("Step — not wired until Phase 1b game loop exists.");
});

/** Optional TDD panel: poll test_status.json written by a watch script. */
async function refreshTdd() {
  try {
    const res = await fetch("./test_status.json", { cache: "no-store" });
    if (!res.ok) return;
    const data = await res.json();
    document.getElementById("tdd-green").textContent = data.green ?? "—";
    document.getElementById("tdd-red").textContent = data.red ?? "—";
    document.getElementById("tdd-last").textContent = data.last_run ?? "—";
    document.getElementById("tdd-chains").textContent = data.chains ?? "—";
  } catch {
    /* file optional during scaffold */
  }
}

setInterval(refreshTdd, 2000);
refreshTdd();
log("Viewer shell ready. Load WASM when push_wasm is built.");
