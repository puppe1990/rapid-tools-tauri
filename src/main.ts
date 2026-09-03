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
  /** Source extensions this tool accepts (dialog + drop + backend). */
  accept: string[];
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
const IMAGE_INPUTS = ["png", "jpg", "jpeg", "webp", "heic", "avif", "enc"];
const VIDEO_INPUTS = ["mp4", "mov", "webm", "mkv", "avi", "3gp"];
const AUDIO_INPUTS = ["mp3", "wav", "ogg", "aac", "flac"];
/** Audio Converter additionally accepts M4A (AAC in MP4 container). */
const AUDIO_CONVERTER_INPUTS = [...AUDIO_INPUTS, "m4a"];

const TOOLS: ToolDef[] = [
  {
    id: "image",
    formats: ["png", "jpg", "webp", "heic", "avif", "enc"],
    accept: IMAGE_INPUTS,
    multi: true,
    accent: "#f97316",
  },
  {
    id: "video",
    formats: ["mp4", "mov", "webm", "mkv", "avi", "3gp"],
    accept: VIDEO_INPUTS,
    multi: true,
    accent: "#6366f1",
  },
  {
    id: "image_resizer",
    formats: ["original", "jpg", "png", "webp"],
    accept: ["png", "jpg", "jpeg", "webp"],
    multi: true,
    accent: "#06b6d4",
  },
  {
    id: "video_compressor",
    formats: ["mp4"],
    accept: VIDEO_INPUTS,
    multi: true,
    accent: "#f43f5e",
  },
  {
    id: "extract_audio",
    formats: ["mp3", "wav", "ogg", "aac", "flac"],
    accept: [...VIDEO_INPUTS, "ts"],
    multi: true,
    accent: "#d946ef",
  },
  {
    id: "audio",
    formats: ["mp3", "wav", "ogg", "aac", "flac"],
    accept: AUDIO_CONVERTER_INPUTS,
    multi: true,
    accent: "#10b981",
  },
  {
    id: "photos_to_pdf",
    formats: ["pdf"],
    accept: IMAGE_INPUTS,
    multi: true,
    accent: "#0ea5e9",
  },
  {
    id: "pdf_to_images",
    formats: ["png", "jpg"],
    accept: ["pdf"],
    multi: false,
    accent: "#8b5cf6",
  },
  {
    id: "together_audios",
    formats: ["mp3", "wav", "ogg", "aac", "flac"],
    accept: AUDIO_INPUTS,
    multi: true,
    accent: "#f59e0b",
  },
  {
    id: "together_videos",
    formats: ["mp4", "mov", "webm", "mkv", "avi", "3gp"],
    accept: VIDEO_INPUTS,
    multi: true,
    accent: "#ec4899",
  },
  {
    id: "images_to_video",
    formats: ["mp4", "gif"],
    accept: ["png", "jpg", "jpeg", "webp"],
    multi: true,
    accent: "#14b8a6",
  },
  {
    id: "qr_reader",
    formats: ["text"],
    accept: ["png", "jpg", "jpeg", "webp"],
    multi: false,
    accent: "#84cc16",
  },
];

let currentTool: ToolDef = TOOLS[0];
let selectedPaths: string[] = [];
let results: ConversionResult[] = [];
let qrText: string | null = null;
let searchQuery = "";
let fileSearchQuery = "";

interface SelectOption {
  value: string;
  label: string;
}

/** Searchable single-select used for multi-option fields (format, fit, preset, …). */
class SearchSelect {
  readonly root: HTMLElement;
  private options: SelectOption[] = [];
  private value = "";
  private open = false;
  private filter = "";
  private readonly trigger: HTMLButtonElement;
  private readonly panel: HTMLElement;
  private readonly searchInput: HTMLInputElement;
  private readonly list: HTMLElement;
  private readonly empty: HTMLElement;

  constructor(root: HTMLElement) {
    this.root = root;
    root.classList.add("search-select");
    root.innerHTML = `
      <button type="button" class="search-select-trigger" aria-haspopup="listbox" aria-expanded="false">
        <span class="search-select-value"></span>
        <svg class="search-select-chevron" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
          <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.17l3.71-3.94a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd"/>
        </svg>
      </button>
      <div class="search-select-panel hidden" role="listbox">
        <div class="search-select-search">
          <input type="search" class="search-select-input" autocomplete="off" />
        </div>
        <div class="search-select-options"></div>
        <p class="search-select-empty hidden"></p>
      </div>
    `;
    this.trigger = root.querySelector(".search-select-trigger") as HTMLButtonElement;
    this.panel = root.querySelector(".search-select-panel") as HTMLElement;
    this.searchInput = root.querySelector(".search-select-input") as HTMLInputElement;
    this.list = root.querySelector(".search-select-options") as HTMLElement;
    this.empty = root.querySelector(".search-select-empty") as HTMLElement;

    this.trigger.addEventListener("click", (e) => {
      e.stopPropagation();
      this.setOpen(!this.open);
    });
    this.searchInput.addEventListener("click", (e) => e.stopPropagation());
    this.searchInput.addEventListener("input", () => {
      this.filter = this.searchInput.value;
      this.renderOptions();
    });
    this.searchInput.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        this.setOpen(false);
        this.trigger.focus();
      }
    });
  }

  setOptions(options: SelectOption[], preferred?: string): void {
    this.options = options;
    const keep =
      preferred && options.some((o) => o.value === preferred)
        ? preferred
        : options.some((o) => o.value === this.value)
          ? this.value
          : (options[0]?.value ?? "");
    this.value = keep;
    this.filter = "";
    this.searchInput.value = "";
    this.renderTrigger();
    this.renderOptions();
  }

  getValue(): string {
    return this.value;
  }

  private setOpen(open: boolean): void {
    this.open = open;
    this.panel.classList.toggle("hidden", !open);
    this.trigger.setAttribute("aria-expanded", open ? "true" : "false");
    this.root.classList.toggle("is-open", open);
    if (open) {
      this.filter = "";
      this.searchInput.value = "";
      this.searchInput.placeholder = t("common.searchOptions");
      this.renderOptions();
      requestAnimationFrame(() => this.searchInput.focus());
    }
  }

  close(): void {
    this.setOpen(false);
  }

  private renderTrigger(): void {
    const label =
      this.options.find((o) => o.value === this.value)?.label ?? this.value ?? "—";
    const valueEl = this.trigger.querySelector(".search-select-value");
    if (valueEl) valueEl.textContent = label;
  }

  private renderOptions(): void {
    const q = this.filter.trim().toLowerCase();
    const filtered = q
      ? this.options.filter(
          (o) => o.label.toLowerCase().includes(q) || o.value.toLowerCase().includes(q),
        )
      : this.options;

    this.list.innerHTML = "";
    this.empty.classList.toggle("hidden", filtered.length > 0);
    this.empty.textContent = t("common.noMatchingOptions");

    for (const opt of filtered) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = `search-select-option${opt.value === this.value ? " is-selected" : ""}`;
      btn.setAttribute("role", "option");
      btn.setAttribute("aria-selected", opt.value === this.value ? "true" : "false");
      btn.textContent = opt.label;
      btn.addEventListener("click", (e) => {
        e.stopPropagation();
        this.value = opt.value;
        this.renderTrigger();
        this.setOpen(false);
      });
      this.list.appendChild(btn);
    }
  }
}

const selectControllers = new Map<string, SearchSelect>();

function getSearchSelect(id: string): SearchSelect {
  let ctrl = selectControllers.get(id);
  if (!ctrl) {
    const root = $(id);
    ctrl = new SearchSelect(root);
    selectControllers.set(id, ctrl);
  }
  return ctrl;
}

function selectValue(id: string): string {
  return selectControllers.get(id)?.getValue() ?? "";
}

function closeAllSearchSelects(except?: HTMLElement): void {
  for (const ctrl of selectControllers.values()) {
    if (except && ctrl.root === except) continue;
    ctrl.close();
  }
}

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

function fileExtension(path: string): string {
  const name = basename(path);
  const dot = name.lastIndexOf(".");
  if (dot <= 0) return "";
  return name.slice(dot + 1).toLowerCase();
}

function canonicalExt(ext: string): string {
  return ext.toLowerCase() === "jpeg" ? "jpg" : ext.toLowerCase();
}

function isAcceptedSource(path: string): boolean {
  const ext = canonicalExt(fileExtension(path));
  if (!ext) return false;
  return currentTool.accept.some((allowed) => canonicalExt(allowed) === ext);
}

function suggestedToolForExt(ext: string): ToolId | null {
  const e = canonicalExt(ext);
  if (["mp4", "mov", "webm", "mkv", "avi", "3gp", "ts"].includes(e)) return "video";
  if (["mp3", "wav", "ogg", "aac", "flac", "m4a"].includes(e)) return "audio";
  if (e === "pdf") return "pdf_to_images";
  if (["png", "jpg", "webp", "heic", "avif", "enc"].includes(e)) return "image";
  return null;
}

function acceptedList(): string {
  return currentTool.accept.map((a) => a.toUpperCase()).join(", ");
}

function suggestionForExt(ext: string): string {
  const suggested = suggestedToolForExt(ext);
  if (suggested && suggested !== currentTool.id) {
    return t("errors.tryTool", { tool: toolText(suggested, "name") });
  }
  return "";
}

function stringifyInvokeError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error);
}

function formatInvokeError(error: unknown): string {
  const raw = stringifyInvokeError(error);
  const [code, ...rest] = raw.split(":");
  const detail = rest.join(":").trim();
  switch (code.trim()) {
    case "unsupported_source_format": {
      const ext = (detail || "unknown").toUpperCase();
      return t("errors.unsupportedSource", {
        ext,
        tool: toolText(currentTool.id, "name"),
        accepted: acceptedList(),
        suggestion: suggestionForExt(detail.toLowerCase()),
      });
    }
    case "ffmpeg_not_found":
      return t("errors.ffmpegMissing");
    case "imagemagick_not_found":
      return t("errors.imagemagickMissing");
    case "ghostscript_not_found":
      return t("errors.ghostscriptMissing");
    case "source_file_not_found":
      return t("errors.sourceMissing");
    case "path_not_found":
      return t("errors.pathMissing");
    case "zbar_unavailable":
      return t("errors.zbarMissing");
    case "no_qr_found":
      return t("errors.noQr");
    case "no_video_stream":
    case "invalid_media_file":
      return t("errors.invalidVideo");
    case "no_audio_stream":
      return t("errors.invalidAudio");
    default:
      if (/ffmpeg: (command )?not found/i.test(raw)) {
        return t("errors.ffmpegMissing");
      }
      return t("errors.conversionFailed");
  }
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
  fileSearchQuery = "";
  closeAllSearchSelects();
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
  const formatCtrl = getSearchSelect("target-format");
  formatCtrl.setOptions(
    currentTool.formats.map((f) => ({ value: f, label: f.toUpperCase() })),
  );

  getSearchSelect("fit").setOptions(
    ["contain", "cover", "stretch"].map((v) => ({ value: v, label: v })),
    "contain",
  );
  getSearchSelect("preset").setOptions(
    ["small", "balanced", "high"].map((v) => ({ value: v, label: v })),
    "balanced",
  );
  getSearchSelect("max-resolution").setOptions(
    ["original", "1080", "720", "480"].map((v) => ({ value: v, label: v })),
    "original",
  );

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
  const q = fileSearchQuery.trim().toLowerCase();
  const indexed = selectedPaths.map((path, index) => ({ path, index }));
  const visible = q
    ? indexed.filter(({ path }) => basename(path).toLowerCase().includes(q))
    : indexed;

  if (n === 0) {
    $("file-summary").textContent = t("common.noFiles");
  } else if (q && visible.length !== n) {
    $("file-summary").textContent = t("common.filesVisible", {
      visible: visible.length,
      total: n,
    });
  } else {
    $("file-summary").textContent = t("common.filesSelected", { count: n });
  }

  $("btn-clear-files").classList.toggle("hidden", n === 0);
  $("file-search-wrap").classList.toggle("hidden", n < 2);
  const searchInput = $("file-search-input") as HTMLInputElement;
  searchInput.placeholder = t("common.searchFiles");
  if (searchInput.value !== fileSearchQuery) {
    searchInput.value = fileSearchQuery;
  }

  const list = $("file-list");
  list.innerHTML = "";
  for (const { path, index } of visible) {
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
    remove.addEventListener("click", (e) => {
      e.stopPropagation();
      selectedPaths = selectedPaths.filter((_, i) => i !== index);
      renderFiles();
      updateRunEnabled();
    });
    li.appendChild(remove);
    list.appendChild(li);
  }

  const empty = $("file-search-empty");
  const showEmpty = n > 0 && visible.length === 0;
  empty.classList.toggle("hidden", !showEmpty);
  empty.textContent = showEmpty ? t("common.noMatchingFiles") : "";
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
        setStatus(formatInvokeError(e), true);
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

function allowsMultipleFiles(): boolean {
  return (
    currentTool.multi &&
    currentTool.id !== "pdf_to_images" &&
    currentTool.id !== "qr_reader"
  );
}

function rejectStatus(rejected: string[]): string {
  const names = rejected.map(basename).join(", ");
  const suggestion =
    suggestionForExt(fileExtension(rejected[0] ?? "")) ||
    t("errors.acceptedInputs", { accepted: acceptedList() });
  return t("errors.filesRejected", {
    count: rejected.length,
    names,
    tool: toolText(currentTool.id, "name"),
    suggestion,
  });
}

/** Merge dropped/picked paths into selection (append for multi tools). */
function addPaths(paths: string[], opts: { replace?: boolean } = {}): void {
  const cleaned = paths
    .map((p) => p.trim())
    .filter((p) => p.length > 0 && !p.endsWith("/") && !p.endsWith("\\"));

  if (!cleaned.length) return;

  const accepted = cleaned.filter(isAcceptedSource);
  const rejected = cleaned.filter((path) => !isAcceptedSource(path));

  if (!accepted.length) {
    setStatus(rejectStatus(rejected), true);
    return;
  }

  const replace = opts.replace === true || !allowsMultipleFiles();
  let next: string[];

  if (replace) {
    next = allowsMultipleFiles() ? accepted : [accepted[0]];
  } else {
    const seen = new Set(selectedPaths);
    next = [...selectedPaths];
    for (const path of accepted) {
      if (!seen.has(path)) {
        seen.add(path);
        next.push(path);
      }
    }
  }

  const added = replace ? next.length : next.length - selectedPaths.length;

  selectedPaths = next;
  renderFiles();
  updateRunEnabled();
  if (rejected.length) {
    setStatus(rejectStatus(rejected), true);
  } else if (added > 0) {
    setStatus(t("common.filesAdded", { count: added }));
  } else {
    setStatus("");
  }
}

function setDropActive(active: boolean): void {
  const zone = $("drop-zone");
  zone.classList.toggle("drag-active", active);
  const hint = document.getElementById("drop-hint");
  if (hint) {
    hint.textContent = t(active ? "common.dropActive" : "common.dropHint");
  }
}

async function pickFiles() {
  const multiple = allowsMultipleFiles();

  const selected = await open({
    multiple,
    directory: false,
    filters: [
      {
        name: toolText(currentTool.id, "name"),
        extensions: currentTool.accept,
      },
    ],
  });

  if (selected === null) return;
  const paths = Array.isArray(selected) ? selected : [selected];
  // Dialog selection replaces for clarity; drop appends for multi tools.
  addPaths(paths, { replace: true });
}

/**
 * Native Tauri drag-and-drop gives real filesystem paths (HTML5 File API does not).
 * Also keep lightweight HTML5 handlers so the drop zone still highlights in browser preview.
 */
async function setupDragAndDrop(): Promise<void> {
  const zone = $("drop-zone");

  // Keyboard / click on the zone opens the file dialog (except interactive controls).
  zone.addEventListener("click", (e) => {
    const target = e.target as HTMLElement;
    if (
      target.closest(
        "button, a, input, select, label, .file-remove, .search-select, .file-search-wrap",
      )
    ) {
      return;
    }
    void pickFiles();
  });
  zone.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      void pickFiles();
    }
  });

  // Prevent the browser from navigating away when files are dropped on the page.
  const preventNav = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };
  window.addEventListener("dragover", preventNav);
  window.addEventListener("drop", preventNav);

  // HTML5 highlight fallback (paths often useless in Tauri webview without native event).
  zone.addEventListener("dragenter", (e) => {
    e.preventDefault();
    setDropActive(true);
  });
  zone.addEventListener("dragover", (e) => {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
    setDropActive(true);
  });
  zone.addEventListener("dragleave", (e) => {
    e.preventDefault();
    // Only clear when leaving the zone itself, not children.
    if (e.relatedTarget instanceof Node && zone.contains(e.relatedTarget)) return;
    setDropActive(false);
  });
  zone.addEventListener("drop", (e) => {
    e.preventDefault();
    setDropActive(false);
    // Prefer Tauri native paths; HTML5 File.path is non-standard and often empty.
    const files = e.dataTransfer?.files;
    if (!files?.length) return;
    const paths: string[] = [];
    for (let i = 0; i < files.length; i++) {
      const file = files[i] as File & { path?: string };
      if (file.path) paths.push(file.path);
    }
    if (paths.length) addPaths(paths);
  });

  try {
    const win = getCurrentWindow();
    await win.onDragDropEvent((event) => {
      const payload = event.payload;
      switch (payload.type) {
        case "enter":
        case "over":
          setDropActive(true);
          break;
        case "leave":
          setDropActive(false);
          break;
        case "drop":
          setDropActive(false);
          if (payload.paths?.length) {
            addPaths(payload.paths);
          }
          break;
      }
    });
  } catch {
    // Non-Tauri environment — HTML5 handlers above still work for highlighting.
  }
}

async function runTool() {
  setLoading(true);
  setStatus(t("common.processing"));
  results = [];
  qrText = null;
  const format = selectValue("target-format");

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
        const fit = selectValue("fit") || "contain";
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
        const preset = selectValue("preset") || "balanced";
        const maxResolution = selectValue("max-resolution") || "original";
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
    setStatus(formatInvokeError(e), true);
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
    setStatus(formatInvokeError(e), true);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  void (async () => {
    await initI18n();
    refreshAllUi();
    void ensureMaximizedOnLaunch();
    await setupDragAndDrop();

    $("btn-pick").addEventListener("click", (e) => {
      e.stopPropagation();
      void pickFiles();
    });
    $("btn-run").addEventListener("click", () => {
      void runTool();
    });
    $("btn-zip").addEventListener("click", () => {
      void zipResults();
    });
    $("btn-clear-files").addEventListener("click", (e) => {
      e.stopPropagation();
      selectedPaths = [];
      fileSearchQuery = "";
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
    $("file-search-input").addEventListener("click", (e) => e.stopPropagation());
    $("file-search-input").addEventListener("keydown", (e) => e.stopPropagation());
    $("file-search-input").addEventListener("input", (e) => {
      e.stopPropagation();
      fileSearchQuery = (e.target as HTMLInputElement).value;
      renderFiles();
    });
    $("btn-locale").addEventListener("click", () => {
      void switchLocale();
    });
    document.addEventListener("click", () => closeAllSearchSelects());
  })();
});
