import { writable } from 'svelte/store';

import en from './en.json';
import zhCN from './zhCN.json';

const translations = {
    en,
    zhCN
};

type Language = 'en' | 'zhCN';

const initialLanguage = (navigator.language.replace('-', '') as Language) || 'en';
export const language = writable<Language>(initialLanguage);

export const t = writable(translations[initialLanguage]);

language.subscribe((lang) => {
    t.set(translations[lang]);
});

/**
 * Translate string with %placeholder%
 * @param translate Translate string
 * @param replacements {"placeholder": "P14C3H01D3R"}
 * @returns Translate string with P14C3H01D3R
 */
export function T(translate: string, replacements: { [key: string]: string | undefined }): string {
    return translate.replace(/%(\w+)%/g, (_, key) => replacements[key] || `%${key}%`);
}
