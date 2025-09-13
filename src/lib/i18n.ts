import { m } from '$lib/paraglide/messages';
import { setLocale, type Locale } from '$lib/paraglide/runtime';

export function ChangeLanguage(lang: Locale | string) {
    setLocale(lang as Locale);
    console.log(`Language changed to ${lang}`);
}

export const LanguageTitle: Record<Locale, () => string> = {
    "en": m.language_en,
    "zh-CN": m.language_zh_CN
}

export const locales = Object.keys(LanguageTitle) as Locale[];


