# RapidTools (Tauri)

Desktop batch media tools built with **Tauri 2** (Rust domain + web UI). Conversion runs locally via **ImageMagick** (`magick`/`convert`) and **ffmpeg** (plus `zbarimg` for QR).

UI layout and tool colors follow the RapidTools Phoenix LiveView product screens.

## Features (product parity)

| Tool             | Formats / notes                                     |
| ---------------- | --------------------------------------------------- |
| Image converter  | png, jpg, webp, heic, avif, enc                     |
| Image resizer    | original / jpg / png / webp · contain/cover/stretch |
| Video converter  | mp4, mov, webm, mkv, avi                            |
| Video compressor | mp4 · presets small/balanced/high                   |
| Extract audio    | mp3, wav, ogg, aac, flac                            |
| Audio converter  | mp3, wav, ogg, aac, flac                            |
| Photos to PDF    | images → pdf                                        |
| PDF to images    | pdf → png/jpg pages                                 |
| Together audios  | join ≥2 audio files                                 |
| Together videos  | join ≥2 videos                                      |
| Images to video  | mp4 / gif                                           |
| QR reader        | decode with `zbarimg`                               |
| ZIP batch        | pack conversion results                             |

### Explicit deferrals

- **Document converter** (DOCX/ODT/RTF/MD/HTML → PDF via LibreOffice, PDF→Markdown fidelity modes): deferred. The Phoenix app uses LibreOffice + Extractous; this remake covers photos↔PDF and image/video/audio batch tools first. Revisit when Office document workflows are required on desktop.

## Requirements

- Rust (`cargo` / `rustc`)
- Node.js + npm
- `ffmpeg` (+ `ffprobe`)
- ImageMagick (`magick` or `convert`)
- `zbarimg` (QR reader)
- `qrencode` (only for generating QR test fixtures)

## App icon

Master artwork: `app-icon.png` (1024×1024). Platform packs live under `src-tauri/icons/` (icns, ico, png sizes, iOS/Android).

Regenerate after editing the master:

```bash
npx tauri icon app-icon.png -o src-tauri/icons
```

## Setup

```bash
cd rapid-tools-tauri
npm install
```

Husky installs a **pre-commit** hook that runs lint-staged (ESLint + Prettier + `rustfmt`), TypeScript, Clippy, and domain tests.

## Run the app

```bash
npm run tauri dev
```

## Quality checks

| Command                               | What it does                                                   |
| ------------------------------------- | -------------------------------------------------------------- |
| `npm run lint`                        | ESLint (TS)                                                    |
| `npm run format` / `format:check`     | Prettier write / check                                         |
| `npm run typecheck`                   | `tsc --noEmit`                                                 |
| `npm run test`                        | Rust domain tests (`cargo test --lib`)                         |
| `npm run clippy`                      | Clippy with `-D warnings`                                      |
| `npm run fmt:rust` / `fmt:rust:check` | `rustfmt`                                                      |
| `npm run check`                       | All of the above (CI-equivalent locally)                       |
| `npm run precommit`                   | Hook entry: lint-staged + typecheck + rustfmt + clippy + tests |

### Tests (TDD / domain)

Conversion logic lives under `src-tauri/src/domain/` and is covered by unit tests that create **real** media fixtures (via `magick`/`ffmpeg`/`qrencode`) and assert real output files.

```bash
npm run test
# or focused:
cd src-tauri && cargo test image_converter
```

### CI

GitHub Actions (`.github/workflows/ci.yml`) runs on every PR and push to `main`:

1. **Frontend** — Prettier, ESLint, TypeScript
2. **Rust** — `rustfmt`, Clippy, `cargo test --lib` (with ffmpeg, ImageMagick, zbar, qrencode)

## Internationalization (i18n)

Frontend uses **i18next** with the same locales as the Phoenix app:

| Locale         | File                          |
| -------------- | ----------------------------- |
| `en` (default) | `src/i18n/locales/en.json`    |
| `pt-BR`        | `src/i18n/locales/pt-BR.json` |

- Detection: `localStorage` → `navigator.language` → `en`
- Toggle: language button at the bottom of the sidebar (like Phoenix)
- Setup: `src/i18n/index.ts` — `initI18n()`, `t()`, `setLocale()`

## Architecture

- **`domain/*`**: pure functions (`convert`, `join`, `build` zip, …) shelling out to local CLIs; tests call these directly.
- **`lib.rs` commands**: thin Tauri `invoke` wrappers + default temp output dirs.
- **Frontend (`src/main.ts`)**: multi-tool sidebar, dialog file pick, format options, results list, ZIP batch.
- **i18n (`src/i18n/`)**: English + Portuguese (BR) UI strings for tools and chrome.

## License

Same intent as the parent RapidTools project; local tooling only.
