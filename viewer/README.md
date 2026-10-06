# Push Debug Table Viewer

Phase 1b visual debugger for the Rust engine (WebAssembly + HTML/CSS/JS).

## Build WASM

From the repo root (requires [wasm-pack](https://rustwasm.github.io/wasm-pack/)):

```bash
wasm-pack build push_wasm --target web --out-dir ../viewer/pkg
```

## Serve locally

Open via a local static server (ES modules need HTTP, not `file://`):

```bash
# from viewer/
npx --yes serve .
# or: python -m http.server 8080
```

Then open the printed URL.

## Assets

- Card rendering: add [selfthinker/CSS-Playing-Cards](https://github.com/selfthinker/CSS-Playing-Cards) under `css-playing-cards/` (submodule or copy) when Epic 1b UI work starts.
- Generated `pkg/` is gitignored.

## TDD panel

If a watch script writes `viewer/test_status.json` with `{ "green", "red", "last_run", "chains" }`, the side panel updates every 2s.
