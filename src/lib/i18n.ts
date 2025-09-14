import { m } from '$lib/paraglide/messages';
import { setLocale, type Locale } from '$lib/paraglide/runtime';
import { toSentenceCase as ToSentenceCase  } from 'js-convert-case';

export function changeLanguage(lang: Locale | string) {
    setLocale(lang as Locale);
    console.log(`Language changed to ${lang}`);
}

export const languageTitle: Record<Locale, () => string> = {
    "en": m.language_en,
    "zh-CN": m.language_zh_CN
}

export function isEnglish(str: string): boolean {
    return /^[a-zA-Z]+$/.test(str);
}

export function toSentenceCase(str: string): string {
    if (isEnglish(str)) {
        return ToSentenceCase(str);
    } else {
        return str; // Return the original string if it's not English
    }
}

export const locales = Object.keys(languageTitle) as Locale[];


