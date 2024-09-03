import { get, writable } from 'svelte/store';

import en from './en.json';
import zhCN from './zhCN.json';
import Languages from './Languages';

const translations = {
    en,
    zhCN
};

export type Language = 'en' | 'zhCN';

const initialLanguage = (navigator.language.replace('-', '') as Language) || 'en';
export const language = writable<Language>(initialLanguage);
export const currentTranslations = writable(translations[initialLanguage]);


export function getLangName(lang: Language | string) 
{
    return Languages[lang as Language];
}

export function setLang(lang: Language) {
    language.set(lang);
}

language.subscribe((lang) => {
    currentTranslations.set(translations[lang]);
});

export function t(key: string) : string
{
    return (get(currentTranslations) as { [key: string]: string })[key] || key;
}

export function getT(key: string, lang: Language) : string
{
    return (translations[lang] as { [key: string]: string })[key] || key;
}

/**
 * Translate string with %placeholder%
 * @param Key Translate Key
 * @param replacements {"placeholder": "P14C3H01D3R"}
 * @returns Translate string with P14C3H01D3R
 */
export function T(key: string, replacements: { [key: string]: any } = {}): string {
    return replaceString(t(key), replacements)
}

export function replaceString(str: string, replacements: { [key: string] : any }): string {
    return str.replace(/%(\w+)%/g, (_, key) => replacements[key] || `%${key}%`).replace(/{(\w+)}/g, (_, key) => replacements[key] || `{${key}}`);
}


