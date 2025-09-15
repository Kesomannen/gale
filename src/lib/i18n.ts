import { m } from '$lib/paraglide/messages';
import { getLocale, setLocale, type Locale } from '$lib/paraglide/runtime';
import { toSentenceCase as ToSentenceCase  } from 'js-convert-case';
import * as api from '$lib/api';
import { locale } from '@tauri-apps/plugin-os';

export function changeLanguage(lang: Locale | string) {
    setLocale(lang as Locale);
    console.log(`Language changed to ${lang}`);
}

export async function checkLanguage()
{
    try
    {
        if (await api.state.isFirstRun()) {
            let lang = await locale()
            if (!lang || !locales.includes(lang)) {
                return;
            }

            let pref = await api.prefs.get();
            pref.language = lang;
            await api.prefs.set(pref);
            changeLanguage(lang);
            return;
        }

        let lang = (await api.prefs.get()).language as Locale;
        if (lang === getLocale()) {
            return;
        }

        changeLanguage(lang);
    }
    catch (e) {
        console.error(e);
    }
    finally {
        console.log(`Language is ${getLocale()}`);
    }
}

export const languageTitle: Record<Locale, () => string> = {
    "en": m.language_en,
    "zh-CN": m.language_zh_CN
}


export function isEnglish(str: string): boolean {
    return /^[a-zA-Z\s]*$/.test(str);
}

export function toSentenceCase(str: string): string {
    if (isEnglish(str)) {
        return ToSentenceCase(str);
    } else {
        return str; // Return the original string if it's not English
    }
}

export const locales = Object.keys(languageTitle);


