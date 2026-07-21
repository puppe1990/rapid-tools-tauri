# RapidTools (Tauri)

Desktop batch media tools built with **Tauri 2** (Rust domain + web UI). Conversion runs locally via **ImageMagick** (`magick`/`convert`) and **ffmpeg** (plus `zbarimg` for QR).

UI layout and tool colors follow the RapidTools Phoenix LiveView product screens.

## Features (product parity)

| Tool | Formats / notes |
|------|-----------------|
| Image converter | png, jpg, webp, heic, avif, enc |
| Image resizer | original / jpg / png / webp · contain/cover/stretch |
| Video converter | mp4, mov, webm, mkv, avi |
| Video compressor | mp4 · presets small/balanced/high |
| Extract audio | mp3, wav, ogg, aac, flac |
| Audio converter | mp3, wav, ogg, aac, flac |
| Photos to PDF | images → pdf |
| PDF to images | pdf → png/jpg pages |
| Together audios | join ≥2 audio files |
| Together videos | join ≥2 videos |
| Images to video | mp4 / gif |
| QR reader | decode with `zbarimg` |
| ZIP batch | pack conversion results |

### Explicit deferrals

- **Document converter** (DOCX/ODT/RTF/MD/HTML → PDF via LibreOffice, PDF→Markdown fidelity modes): deferred. The Phoenix app uses LibreOffice + Extractous; this remake covers photos↔PDF and image/video/audio batch tools first. Revisit when Office document workflows are required on desktop.

## Requirements

- Rust (`cargo` / `rustc`)
- Node.js + npm
- `ffmpeg` (+ `ffprobe`)
- ImageMagick (`magick` or `convert`)
- `zbarimg` (QR reader)
- `qrencode` (only for generating QR test fixtures)

## Setup

```bash
cd rapid-tools-tauri
npm install
```

## Run the app

```bash
npm run tauri dev
```

Or:

```bash
cd src-tauri && cargo build
# then from repo root:
npm run tauri dev
```

## Tests (TDD / domain)

Conversion logic lives under `src-tauri/src/domain/` and is covered by unit tests that create **real** media fixtures (via `magick`/`ffmpeg`/`qrencode`) and assert real output files.

```bash
cd src-tauri
cargo test
```

Focused examples:

```bash
cargo test image_converter
cargo test video_converter
cargo test audio_converter
cargo test zip_archive
```

## Architecture

- **`domain/*`**: pure functions (`convert`, `join`, `build` zip, …) shelling out to local CLIs; tests call these directly.
- **`lib.rs` commands**: thin Tauri `invoke` wrappers + default temp output dirs.
- **Frontend (`src/main.ts`)**: multi-tool sidebar, dialog file pick, format options, results list, ZIP batch.

## License

Same intent as the parent RapidTools project; local tooling only.
