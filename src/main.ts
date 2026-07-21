import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

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
  name: string;
  blurb: string;
  description: string;
  hint: string;
  badge: string;
  acceptHelp: string;
  formats: string[];
  multi: boolean;
  /** CSS color for nav dot / card accent (matches Phoenix ToolNavigation) */
  accent: string;
  runLabel: string;
  loadingTitle: string;
  emptyCopy: string;
}

interface ConversionResult {
  output_path: string;
  filename: string;
  media_type: string;
  target_format: string;
}

/** Tool metadata aligned with Phoenix ToolNavigation + LiveView copy. */
const TOOLS: ToolDef[] = [
  {
    id: "image",
    name: "Image Converter",
    blurb: "Batch image conversion",
    description:
      "Converta imagens para PNG, JPG, WEBP, HEIC, AVIF e ENC com downloads individuais ou em lote.",
    hint: "Ideal para exportar assets para web, social, aplicativos e bibliotecas de design.",
    badge: "Image workflow",
    acceptHelp: "Entradas aceitas: JPG, JPEG, PNG, WEBP, HEIC, AVIF e ENC.",
    formats: ["png", "jpg", "webp", "heic", "avif", "enc"],
    multi: true,
    accent: "#f97316",
    runLabel: "Converter imagens",
    loadingTitle: "Convertendo imagens",
    emptyCopy:
      "Envie várias imagens, escolha o formato final e baixe cada arquivo convertido ou um ZIP com tudo junto.",
  },
  {
    id: "video",
    name: "Video Converter",
    blurb: "Convert MP4, MOV, WEBM, MKV and AVI",
    description:
      "Converta vídeos entre MP4, MOV, WEBM, MKV e AVI com downloads individuais ou em lote.",
    hint: "Útil para padronizar formatos de edição, web e compartilhamento.",
    badge: "Video workflow",
    acceptHelp: "Entradas aceitas: MP4, MOV, WEBM, MKV e AVI.",
    formats: ["mp4", "mov", "webm", "mkv", "avi"],
    multi: true,
    accent: "#6366f1",
    runLabel: "Converter vídeos",
    loadingTitle: "Convertendo vídeos",
    emptyCopy:
      "Envie vários vídeos, escolha o container final e baixe cada saída ou um ZIP com o lote.",
  },
  {
    id: "image_resizer",
    name: "Image Resizer",
    blurb: "Resize for social, stores and thumbnails",
    description:
      "Redimensione imagens com encaixe contain, cover ou stretch e exporte no formato desejado.",
    hint: "Presets mentais para social, lojas e thumbnails.",
    badge: "Resize workflow",
    acceptHelp: "Entradas aceitas: JPG, JPEG, PNG e WEBP.",
    formats: ["original", "jpg", "png", "webp"],
    multi: true,
    accent: "#06b6d4",
    runLabel: "Redimensionar imagens",
    loadingTitle: "Redimensionando imagens",
    emptyCopy:
      "Defina largura, altura e encaixe; baixe cada resultado ou o pacote ZIP.",
  },
  {
    id: "video_compressor",
    name: "Video Compressor",
    blurb: "Reduce file size for sharing and upload",
    description:
      "Comprima vídeos para MP4 com presets de qualidade e limite de resolução.",
    hint: "Ideal para reduzir peso antes de enviar ou publicar.",
    badge: "Compress workflow",
    acceptHelp: "Entradas aceitas: MP4, MOV, WEBM, MKV e AVI.",
    formats: ["mp4"],
    multi: true,
    accent: "#f43f5e",
    runLabel: "Comprimir vídeos",
    loadingTitle: "Comprimindo vídeos",
    emptyCopy: "Escolha preset e resolução; baixe o MP4 comprimido ou o ZIP do lote.",
  },
  {
    id: "extract_audio",
    name: "Extract Audio from Video",
    blurb: "Pull MP3, WAV, OGG, AAC and FLAC from video",
    description:
      "Extraia a faixa de áudio de vídeos para MP3, WAV, OGG, AAC ou FLAC.",
    hint: "Rápido para podcasts, samples e legendagem offline.",
    badge: "Extract workflow",
    acceptHelp: "Entradas aceitas: MP4, MOV, WEBM, MKV, AVI e TS.",
    formats: ["mp3", "wav", "ogg", "aac", "flac"],
    multi: true,
    accent: "#d946ef",
    runLabel: "Extrair áudio",
    loadingTitle: "Extraindo áudio",
    emptyCopy: "Envie vídeos e baixe cada áudio extraído ou o ZIP do lote.",
  },
  {
    id: "audio",
    name: "Audio Converter",
    blurb: "Convert MP3, WAV, OGG, AAC and FLAC",
    description:
      "Converta áudios entre MP3, WAV, OGG, AAC e FLAC com downloads individuais ou em lote.",
    hint: "Padronize bibliotecas de som e assets de produto.",
    badge: "Audio workflow",
    acceptHelp: "Entradas aceitas: MP3, WAV, OGG, AAC e FLAC.",
    formats: ["mp3", "wav", "ogg", "aac", "flac"],
    multi: true,
    accent: "#10b981",
    runLabel: "Converter áudios",
    loadingTitle: "Convertendo áudios",
    emptyCopy:
      "Envie vários áudios, escolha o formato final e baixe cada arquivo ou o ZIP.",
  },
  {
    id: "photos_to_pdf",
    name: "Photos to PDF",
    blurb: "Reorder images and export a single PDF",
    description: "Combine fotos em um único PDF para compartilhar ou imprimir.",
    hint: "Útil para relatórios, portfólios e documentos simples.",
    badge: "PDF workflow",
    acceptHelp: "Entradas aceitas: JPG, PNG, WEBP, HEIC, AVIF e ENC.",
    formats: ["pdf"],
    multi: true,
    accent: "#0ea5e9",
    runLabel: "Gerar PDF",
    loadingTitle: "Gerando PDF",
    emptyCopy: "Selecione as imagens na ordem desejada e exporte o PDF.",
  },
  {
    id: "pdf_to_images",
    name: "PDF to images",
    blurb: "Rasterize PDF pages to PNG or JPG",
    description: "Converta páginas de PDF em imagens PNG ou JPG.",
    hint: "Parte do fluxo de documentos do RapidTools.",
    badge: "Document workflow",
    acceptHelp: "Entrada aceita: PDF.",
    formats: ["png", "jpg"],
    multi: false,
    accent: "#8b5cf6",
    runLabel: "Converter PDF",
    loadingTitle: "Convertendo PDF",
    emptyCopy: "Envie um PDF e baixe cada página rasterizada ou o ZIP.",
  },
  {
    id: "together_audios",
    name: "Together Audios",
    blurb: "Join multiple audio files into one track",
    description: "Una dois ou mais áudios em um único arquivo de saída.",
    hint: "Referência de layout mais rico do produto Phoenix.",
    badge: "Join audio",
    acceptHelp: "Selecione pelo menos 2 áudios (MP3, WAV, OGG, AAC, FLAC).",
    formats: ["mp3", "wav", "ogg", "aac", "flac"],
    multi: true,
    accent: "#f59e0b",
    runLabel: "Juntar áudios",
    loadingTitle: "Juntando áudios",
    emptyCopy: "Escolha o formato final e baixe a faixa unificada.",
  },
  {
    id: "together_videos",
    name: "Together Videos",
    blurb: "Join multiple video files into one track",
    description: "Una dois ou mais vídeos em um único arquivo de saída.",
    hint: "Concatenação local via ffmpeg.",
    badge: "Join video",
    acceptHelp: "Selecione pelo menos 2 vídeos (MP4, MOV, WEBM, MKV, AVI).",
    formats: ["mp4", "mov", "webm", "mkv", "avi"],
    multi: true,
    accent: "#ec4899",
    runLabel: "Juntar vídeos",
    loadingTitle: "Juntando vídeos",
    emptyCopy: "Escolha o container final e baixe o vídeo unificado.",
  },
  {
    id: "images_to_video",
    name: "Images to Video",
    blurb: "Turn photos into MP4 or GIF",
    description: "Transforme uma sequência de imagens em MP4 ou GIF animado.",
    hint: "Configure o intervalo entre frames.",
    badge: "Slideshow workflow",
    acceptHelp: "Entradas aceitas: PNG, JPG, JPEG e WEBP.",
    formats: ["mp4", "gif"],
    multi: true,
    accent: "#14b8a6",
    runLabel: "Criar vídeo",
    loadingTitle: "Criando vídeo",
    emptyCopy: "Envie as imagens na ordem e exporte MP4 ou GIF.",
  },
  {
    id: "qr_reader",
    name: "QR Reader",
    blurb: "Decode QR codes from images or camera",
    description: "Decodifique QR codes a partir de arquivos de imagem com zbarimg.",
    hint: "No desktop, use upload de imagem (câmera do browser fica no app web).",
    badge: "QR workflow",
    acceptHelp: "Entradas aceitas: JPG, JPEG, PNG e WEBP.",
    formats: ["text"],
    multi: false,
    accent: "#84cc16",
    runLabel: "Ler QR",
    loadingTitle: "Lendo QR",
    emptyCopy: "Envie uma imagem com QR para ver o payload decodificado.",
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
    const hay = `${tool.name} ${tool.blurb}`.toLowerCase();
    if (q && !hay.includes(q)) continue;

    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = `nav-card${tool.id === currentTool.id ? " active" : ""}`;
    btn.style.setProperty("--nav-accent", tool.accent);
    btn.dataset.searchText = hay;
    btn.innerHTML = `
      <div class="nav-card-row">
        <span class="nav-dot"></span>
        <p class="nav-name">${escapeHtml(tool.name)}</p>
      </div>
      <p class="nav-blurb">${escapeHtml(tool.blurb)}</p>
    `;
    btn.addEventListener("click", () => selectTool(tool.id));
    nav.appendChild(btn);
  }
}

function selectTool(id: ToolId) {
  currentTool = TOOLS.find((t) => t.id === id) ?? TOOLS[0];
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
  $("tool-badge").textContent = currentTool.badge;
  $("tool-title").textContent = currentTool.name;
  $("tool-description").textContent = currentTool.description;
  $("tool-hint").textContent = currentTool.hint;
  $("accept-help").textContent = currentTool.acceptHelp;
  $("formats-list").textContent = currentTool.formats
    .map((f) => f.toUpperCase())
    .join(", ");
  $("empty-copy").textContent = currentTool.emptyCopy;
  $("run-label").textContent = currentTool.runLabel;
  $("loading-title").textContent = currentTool.loadingTitle;
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
    ? `${n} arquivo(s) selecionado(s)`
    : "Nenhum arquivo selecionado";
  $("btn-clear-files").classList.toggle("hidden", n === 0);

  const list = $("file-list");
  list.innerHTML = "";
  selectedPaths.forEach((path, index) => {
    const li = document.createElement("li");
    li.innerHTML = `
      <span class="file-name" title="${escapeHtml(path)}">${escapeHtml(basename(path))}</span>
      <span class="file-meta">pronto</span>
    `;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "file-remove";
    remove.setAttribute("aria-label", `Remover ${basename(path)}`);
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
    $("results-kicker").textContent = "QR decodificado";
    ($("btn-zip") as HTMLButtonElement).disabled = true;
    const li = document.createElement("li");
    li.innerHTML = `
      <strong>Payload</strong>
      <span class="path">${escapeHtml(qrText)}</span>
    `;
    list.appendChild(li);
    return;
  }

  if (!results.length) {
    filled.classList.add("hidden");
    empty.classList.remove("hidden");
    ($("btn-zip") as HTMLButtonElement).disabled = true;
    return;
  }

  empty.classList.add("hidden");
  filled.classList.remove("hidden");
  $("results-kicker").textContent = `${results.length} arquivo(s) convertido(s)`;
  ($("btn-zip") as HTMLButtonElement).disabled = false;

  for (const r of results) {
    const li = document.createElement("li");
    li.innerHTML = `
      <strong>${escapeHtml(r.filename)}</strong>
      <span class="result-meta">Saída em ${escapeHtml(r.target_format.toUpperCase())}</span>
      <span class="path">${escapeHtml(r.output_path)}</span>
    `;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "reveal-btn";
    btn.textContent = "Revelar no Finder";
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
    currentTool.id === "together_audios" || currentTool.id === "together_videos"
      ? 2
      : 1;
  const ok = multiJoin
    ? selectedPaths.length >= min
    : selectedPaths.length >= 1;
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
  setStatus("Processando…");
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
        ? "QR decodificado."
        : `Pronto — ${results.length} arquivo(s) de saída.`,
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
  setStatus("Gerando ZIP…");
  try {
    const zip = await invoke<{ path: string; filename: string }>("build_zip", {
      id: `${Date.now()}`,
      entries: results.map((r) => ({
        path: r.output_path,
        filename: r.filename,
      })),
    });
    setStatus(`ZIP pronto: ${zip.path}`);
    await invoke("reveal_path", { path: zip.path });
  } catch (e) {
    setStatus(String(e), true);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  applyTheme();
  renderNav();
  renderToolHeader();
  renderOptions();
  renderFiles();
  renderResults();
  updateRunEnabled();

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
});
