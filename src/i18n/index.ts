import i18n from "i18next";
import en from "./locales/en.json";
import ptBR from "./locales/pt-BR.json";

export const SUPPORTED_LOCALES = ["en", "pt-BR"] as const;
export type AppLocale = (typeof SUPPORTED_LOCALES)[number];

export const LOCALE_STORAGE_KEY = "rapid-tools.locale";

/** Match Phoenix RapidTools: en + pt_BR (here as pt-BR for BCP 47). */
export function normalizeLocale(raw: string | null | undefined): AppLocale {
  if (!raw) return "en";
  const v = raw.replace("_", "-").toLowerCase();
  if (v.startsWith("pt")) return "pt-BR";
  if (v.startsWith("en")) return "en";
  return "en";
}

export function detectLocale(): AppLocale {
  try {
    const stored = localStorage.getItem(LOCALE_STORAGE_KEY);
    if (stored) return normalizeLocale(stored);
  } catch {
    // ignore
  }
  if (typeof navigator !== "undefined" && navigator.language) {
    return normalizeLocale(navigator.language);
  }
  return "en";
}

export function toggleLocale(current: string): AppLocale {
  return normalizeLocale(current) === "en" ? "pt-BR" : "en";
}

export function htmlLang(locale: AppLocale): string {
  return locale === "pt-BR" ? "pt-BR" : "en";
}

export async function initI18n(locale?: AppLocale): Promise<typeof i18n> {
  const lng = locale ?? detectLocale();

  if (!i18n.isInitialized) {
    await i18n.init({
      lng,
      fallbackLng: "en",
      supportedLngs: [...SUPPORTED_LOCALES],
      resources: {
        en: { translation: en },
        "pt-BR": { translation: ptBR },
      },
      interpolation: {
        escapeValue: false,
      },
      returnNull: false,
    });
  } else {
    await i18n.changeLanguage(lng);
  }

  try {
    localStorage.setItem(LOCALE_STORAGE_KEY, lng);
  } catch {
    // ignore
  }

  if (typeof document !== "undefined") {
    document.documentElement.lang = htmlLang(lng);
  }

  return i18n;
}

export function t(key: string, options?: Record<string, unknown>): string {
  return i18n.t(key, options);
}

export function currentLocale(): AppLocale {
  return normalizeLocale(i18n.language);
}

export async function setLocale(locale: AppLocale): Promise<void> {
  const lng = normalizeLocale(locale);
  await i18n.changeLanguage(lng);
  try {
    localStorage.setItem(LOCALE_STORAGE_KEY, lng);
  } catch {
    // ignore
  }
  document.documentElement.lang = htmlLang(lng);
}

export default i18n;
