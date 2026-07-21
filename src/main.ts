import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import {
  currentLocale,
  initI18n,
  setLocale,
  t,
  toggleLocale,
  type AppLocale,
} from "./i18n";

/** Maximize the desktop window once the UI is ready (backup for config/setup). */
async function ensureMaximizedOnLaunch(): Promise<void> {
  try {
    const win = getCurrentWindow();
    if (!(await win.isMaximized())) {
      await win.maximize();
    }
  } catch {
    // Browser preview / non-Tauri shell — ignore.
  }
}

type ToolId =
  | "image"
  | "image_resizer"
  | "video"
  | "video_compressor"
  | "extract_audio"
  | "audio"
  | "photos_to_pdf"
  | "pdf_to_images"
  | "together_audios"
  | "together_videos"
  | "images_to_video"
  | "qr_reader";

interface ToolDef {
  id: ToolId;
  formats: string[];
  multi: boolean;
  /** CSS color for nav dot / card accent (matches Phoenix ToolNavigation) */
  accent: string;
}

interface ConversionResult {
  output_path: string;
  filename: string;
  media_type: string;
  target_format: string;
}

/** Tool registry — copy comes from i18n keys tools.<id>.* */
const TOOLS: ToolDef[] = [
  {
    id: "image",
    formats: ["png", "jpg", "webp", "heic", "avif", "enc"],
    multi: true,
    accent: "#f97316",
  },
  {
    id: "video",
    formats: ["mp4", "mov", "webm", "mkv", "avi"],
    multi: true,
    accent: "#6366f1",
  },
  {
    id: "image_resizer",
    formats: ["original", "jpg", "png", "webp"],
    multi: true,
    accent: "#06b6d4",
  },
  {
    id: "video_compressor",
    formats: ["mp4"],
    multi: true,
    accent: "#f43f5e",
  },
  {
    id: "extract_audio",
    formats: ["mp3", "wav", "ogg", "aac", "flac"],
    multi: true,
    accent: "#d946ef",
  },
  {
    id: "audio",
    formats: ["mp3", "wav", "ogg", "aac", "flac"],
    multi: true,
    accent: "#10b981",
  },
  {
    id: "photos_to_pdf",
    formats: ["pdf"],
    multi: true,
    accent: "#0ea5e9",
  },
  {
    id: "pdf_to_images",
    formats: ["png", "jpg"],
    multi: false,
    accent: "#8b5cf6",
  },
  {
    id: "together_audios",
    formats: ["mp3", "wav", "ogg", "aac", "flac"],
    multi: true,
    accent: "#f59e0b",
  },
  {
    id: "together_videos",
    formats: ["mp4", "mov", "webm", "mkv", "avi"],
    multi: true,
    accent: "#ec4899",
  },
  {
    id: "images_to_video",
    formats: ["mp4", "gif"],
    multi: true,
    accent: "#14b8a6",
  },
  {
    id: "qr_reader",
    formats: ["text"],
    multi: false,
    accent: "#84cc16",
  },
];

let currentTool: ToolDef = TOOLS[0];
let selectedPaths: string[] = [];
let results: ConversionResult[] = [];
let qrText: string | null = null;
let searchQuery = "";

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing #${id}`);
  return el;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function basename(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}

function toolKey(id: ToolId, field: string): string {
  return `tools.${id}.${field}`;
}

function toolText(id: ToolId, field: string): string {
  return t(toolKey(id, field));
}

/** Apply data-i18n / data-i18n-placeholder / data-i18n-aria-label attributes. */
function applyStaticI18n(): void {
  document.querySelectorAll<HTMLElement>("[data-i18n]").forEach((el) => {
    const key = el.dataset.i18n;
    if (key) el.textContent = t(key);
  });
  document.querySelectorAll<HTMLInputElement>("[data-i18n-placeholder]").forEach((el) => {
    const key = el.dataset.i18nPlaceholder;
    if (key) el.placeholder = t(key);
  });
  document.querySelectorAll<HTMLElement>("[data-i18n-aria-label]").forEach((el) => {
    const key = el.dataset.i18nAriaLabel;
    if (key) el.setAttribute("aria-label", t(key));
  });
  document.title = t("app.title");
  updateLocaleButton();
}

function updateLocaleButton(): void {
  const next = toggleLocale(currentLocale());
  const nextName = next === "en" ? t("app.english") : t("app.portuguese");
  $("locale-btn-label").textContent = nextName;
  $("btn-locale").setAttribute("title", t("app.switchTo", { name: nextName }));
  $("btn-locale").setAttribute("aria-label", t("app.switchTo", { name: nextName }));
}

function applyTheme() {
  const section = $("app-section");
  section.className = `app-section theme-${currentTool.id}`;
  $("sidebar-kicker").style.color = currentTool.accent;
}

function renderNav() {
  const nav = $("tool-nav");
  nav.innerHTML = "";
  const q = searchQuery.trim().toLowerCase();

  for (const tool of TOOLS) {
    const name = toolText(tool.id, "name");
    const blurb = toolText(tool.id, "blurb");
    const hay = `${name} ${blurb}`.toLowerCase();
    if (q && !hay.includes(q)) continue;

    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = `nav-card${tool.id === currentTool.id ? " active" : ""}`;
    btn.style.setProperty("--nav-accent", tool.accent);
    btn.dataset.searchText = hay;
    btn.innerHTML = `
      <div class="nav-card-row">
        <span class="nav-dot"></span>
        <p class="nav-name">${escapeHtml(name)}</p>
      </div>
      <p class="nav-blurb">${escapeHtml(blurb)}</p>
    `;
    btn.addEventListener("click", () => selectTool(tool.id));
    nav.appendChild(btn);
  }
}

function selectTool(id: ToolId) {
  currentTool = TOOLS.find((tool) => tool.id === id) ?? TOOLS[0];
  selectedPaths = [];
  results = [];
  qrText = null;
  applyTheme();
  renderNav();
  renderToolHeader();
  renderOptions();
  renderFiles();
  renderResults();
  updateRunEnabled();
  setStatus("");
  setLoading(false);
}

function renderToolHeader() {
  const id = currentTool.id;
  $("tool-badge").textContent = toolText(id, "badge");
  $("tool-title").textContent = toolText(id, "name");
  $("tool-description").textContent = toolText(id, "description");
  $("tool-hint").textContent = toolText(id, "hint");
  $("accept-help").textContent = toolText(id, "acceptHelp");
  $("formats-list").textContent = currentTool.formats
    .map((f) => f.toUpperCase())
    .join(", ");
  // empty-copy uses tool-specific text when empty results shown
  $("empty-copy").textContent = toolText(id, "emptyCopy");
  $("run-label").textContent = toolText(id, "runLabel");
  $("loading-title").textContent = toolText(id, "loadingTitle");
}

function renderOptions() {
  const formatSelect = $("target-format") as HTMLSelectElement;
  formatSelect.innerHTML = "";
  for (const f of currentTool.formats) {
    const opt = document.createElement("option");
    opt.value = f;
    opt.textContent = f.toUpperCase();
    formatSelect.appendChild(opt);
  }

  const show = (id: string, on: boolean) => {
    $(id).classList.toggle("hidden", !on);
  };

  show("width-label", currentTool.id === "image_resizer");
  show("height-label", currentTool.id === "image_resizer");
  show("fit-label", currentTool.id === "image_resizer");
  show("preset-label", currentTool.id === "video_compressor");
  show("resolution-label", currentTool.id === "video_compressor");
  show("interval-label", currentTool.id === "images_to_video");
  show("format-label", currentTool.id !== "qr_reader");
}

function renderFiles() {
  const n = selectedPaths.length;
  $("file-summary").textContent = n
    ? t("common.filesSelected", { count: n })
    : t("common.noFiles");
  $("btn-clear-files").classList.toggle("hidden", n === 0);

  const list = $("file-list");
  list.innerHTML = "";
  selectedPaths.forEach((path, index) => {
    const li = document.createElement("li");
    li.innerHTML = `
      <span class="file-name" title="${escapeHtml(path)}">${escapeHtml(basename(path))}</span>
      <span class="file-meta">${escapeHtml(t("common.ready"))}</span>
    `;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "file-remove";
    remove.setAttribute("aria-label", t("common.removeFile", { name: basename(path) }));
    remove.textContent = "X";
    remove.addEventListener("click", () => {
      selectedPaths = selectedPaths.filter((_, i) => i !== index);
      renderFiles();
      updateRunEnabled();
    });
    li.appendChild(remove);
    list.appendChild(li);
  });
}

function renderResults() {
  const filled = $("results-filled");
  const empty = $("results-empty");
  const list = $("results");
  list.innerHTML = "";

  if (qrText !== null) {
    empty.classList.add("hidden");
    filled.classList.remove("hidden");
    $("results-kicker").textContent = t("common.qrDecoded");
    ($("btn-zip") as HTMLButtonElement).disabled = true;
    const li = document.createElement("li");
    li.innerHTML = `
      <strong>${escapeHtml(t("common.payload"))}</strong>
      <span class="path">${escapeHtml(qrText)}</span>
    `;
    list.appendChild(li);
    return;
  }

  if (!results.length) {
    filled.classList.add("hidden");
    empty.classList.remove("hidden");
    $("empty-kicker").textContent = t("common.batchReady");
    $("empty-copy").textContent = toolText(currentTool.id, "emptyCopy");
    ($("btn-zip") as HTMLButtonElement).disabled = true;
    return;
  }

  empty.classList.add("hidden");
  filled.classList.remove("hidden");
  $("results-kicker").textContent = t("common.convertedCount", {
    count: results.length,
  });
  ($("btn-zip") as HTMLButtonElement).disabled = false;

  for (const r of results) {
    const li = document.createElement("li");
    li.innerHTML = `
      <strong>${escapeHtml(r.filename)}</strong>
      <span class="result-meta">${escapeHtml(
        t("common.outputIn", { format: r.target_format.toUpperCase() }),
      )}</span>
      <span class="path">${escapeHtml(r.output_path)}</span>
    `;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "reveal-btn";
    btn.textContent = t("common.reveal");
    btn.addEventListener("click", async () => {
      try {
        await invoke("reveal_path", { path: r.output_path });
      } catch (e) {
        setStatus(String(e), true);
      }
    });
    li.appendChild(btn);
    list.appendChild(li);
  }
}

function updateRunEnabled() {
  const btn = $("btn-run") as HTMLButtonElement;
  const multiJoin =
    currentTool.id === "together_audios" ||
    currentTool.id === "together_videos" ||
    currentTool.id === "images_to_video" ||
    currentTool.id === "photos_to_pdf";
  const min =
    currentTool.id === "together_audios" || currentTool.id === "together_videos" ? 2 : 1;
  const ok = multiJoin ? selectedPaths.length >= min : selectedPaths.length >= 1;
  btn.disabled = !ok;
}

function setStatus(msg: string, isError = false) {
  const el = $("status");
  el.textContent = msg;
  el.classList.toggle("error", isError && !!msg);
  el.classList.toggle("ok", !isError && !!msg);
}

function setLoading(on: boolean) {
  $("loading-overlay").classList.toggle("hidden", !on);
  $("run-spinner").classList.toggle("hidden", !on);
  if (on) {
    ($("btn-run") as HTMLButtonElement).disabled = true;
  } else {
    updateRunEnabled();
  }
}

function refreshAllUi(): void {
  applyStaticI18n();
  applyTheme();
  renderNav();
  renderToolHeader();
  renderOptions();
  renderFiles();
  renderResults();
  updateRunEnabled();
}

async function switchLocale(): Promise<void> {
  const next = toggleLocale(currentLocale()) as AppLocale;
  await setLocale(next);
  refreshAllUi();
}

async function pickFiles() {
  const multiple =
    currentTool.multi &&
    currentTool.id !== "pdf_to_images" &&
    currentTool.id !== "qr_reader";

  const selected = await open({
    multiple,
    directory: false,
  });

  if (selected === null) return;
  selectedPaths = Array.isArray(selected) ? selected : [selected];
  renderFiles();
  updateRunEnabled();
  setStatus("");
}

async function runTool() {
  setLoading(true);
  setStatus(t("common.processing"));
  results = [];
  qrText = null;
  const format = ($("target-format") as HTMLSelectElement).value;

  try {
    switch (currentTool.id) {
      case "image": {
        for (const path of selectedPaths) {
          results.push(
            await invoke<ConversionResult>("convert_image", {
              sourcePath: path,
              targetFormat: format,
            }),
          );
        }
        break;
      }
      case "image_resizer": {
        const width = Number(($("width") as HTMLInputElement).value);
        const height = Number(($("height") as HTMLInputElement).value);
        const fit = ($("fit") as HTMLSelectElement).value;
        for (const path of selectedPaths) {
          results.push(
            await invoke<ConversionResult>("resize_image", {
              sourcePath: path,
              width,
              height,
              targetFormat: format === "original" ? "original" : format,
              fit,
            }),
          );
        }
        break;
      }
      case "video": {
        for (const path of selectedPaths) {
          results.push(
            await invoke<ConversionResult>("convert_video", {
              sourcePath: path,
              targetFormat: format,
              orientation: "original",
            }),
          );
        }
        break;
      }
      case "video_compressor": {
        const preset = ($("preset") as HTMLSelectElement).value;
        const maxResolution = ($("max-resolution") as HTMLSelectElement).value;
        for (const path of selectedPaths) {
          results.push(
            await invoke<ConversionResult>("compress_video", {
              sourcePath: path,
              preset,
              maxResolution,
              mute: false,
            }),
          );
        }
        break;
      }
      case "extract_audio": {
        for (const path of selectedPaths) {
          results.push(
            await invoke<ConversionResult>("extract_audio", {
              sourcePath: path,
              targetFormat: format,
            }),
          );
        }
        break;
      }
      case "audio": {
        for (const path of selectedPaths) {
          results.push(
            await invoke<ConversionResult>("convert_audio", {
              sourcePath: path,
              targetFormat: format,
            }),
          );
        }
        break;
      }
      case "photos_to_pdf": {
        results.push(
          await invoke<ConversionResult>("photos_to_pdf", {
            sourcePaths: selectedPaths,
          }),
        );
        break;
      }
      case "pdf_to_images": {
        const pages = await invoke<ConversionResult[]>("pdf_to_images", {
          sourcePath: selectedPaths[0],
          targetFormat: format,
        });
        results.push(...pages);
        break;
      }
      case "together_audios": {
        results.push(
          await invoke<ConversionResult>("join_audios", {
            sourcePaths: selectedPaths,
            targetFormat: format,
          }),
        );
        break;
      }
      case "together_videos": {
        results.push(
          await invoke<ConversionResult>("join_videos", {
            sourcePaths: selectedPaths,
            targetFormat: format,
          }),
        );
        break;
      }
      case "images_to_video": {
        const intervalSecs = Number(($("interval") as HTMLInputElement).value);
        results.push(
          await invoke<ConversionResult>("images_to_video", {
            sourcePaths: selectedPaths,
            targetFormat: format,
            intervalSecs,
          }),
        );
        break;
      }
      case "qr_reader": {
        qrText = await invoke<string>("read_qr", {
          sourcePath: selectedPaths[0],
        });
        break;
      }
    }
    setStatus(
      qrText !== null
        ? t("common.qrDecodedStatus")
        : t("common.doneCount", { count: results.length }),
    );
    renderResults();
  } catch (e) {
    setStatus(String(e), true);
    renderResults();
  } finally {
    setLoading(false);
  }
}

async function zipResults() {
  if (!results.length) return;
  setStatus(t("common.buildingZip"));
  try {
    const zip = await invoke<{ path: string; filename: string }>("build_zip", {
      id: `${Date.now()}`,
      entries: results.map((r) => ({
        path: r.output_path,
        filename: r.filename,
      })),
    });
    setStatus(t("common.zipReady", { path: zip.path }));
    await invoke("reveal_path", { path: zip.path });
  } catch (e) {
    setStatus(String(e), true);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  void (async () => {
    await initI18n();
    refreshAllUi();
    void ensureMaximizedOnLaunch();

    $("btn-pick").addEventListener("click", () => {
      void pickFiles();
    });
    $("btn-run").addEventListener("click", () => {
      void runTool();
    });
    $("btn-zip").addEventListener("click", () => {
      void zipResults();
    });
    $("btn-clear-files").addEventListener("click", () => {
      selectedPaths = [];
      renderFiles();
      updateRunEnabled();
      setStatus("");
    });
    $("btn-clear-results").addEventListener("click", () => {
      results = [];
      qrText = null;
      renderResults();
      setStatus("");
    });
    $("tool-search-input").addEventListener("input", (e) => {
      searchQuery = (e.target as HTMLInputElement).value;
      renderNav();
    });
    $("btn-locale").addEventListener("click", () => {
      void switchLocale();
    });
  })();
});
