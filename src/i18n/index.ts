import { get, writable } from 'svelte/store';

import en from './en.json';
import zhCN from './zhCN.json';
import Languages from './Languages';
import { invokeCommand } from '$lib/invoke';
import type { Prefs } from '$lib/models';

const translations = {
    en,
    zhCN
};

export type Language = 'en' | 'zhCN';


export let InitLang : Language;
export const language = writable<Language>(await getLangFormPrefs());
export const currentTranslations = writable(translations[get(language)]);


export function getLangName(lang: Language | string) 
{
    return Languages[lang as Language];
}

export function setLang(lang: Language | string) {
    language.set(lang as Language);
}

/* language.subscribe((lang) => {
    currentTranslations.set(translations[lang]);
});
*/

export async function getLangFormPrefs() : Promise<Language>
{
    var is_first_run =  await invokeCommand<boolean>('is_first_run');
    var Prefs = await invokeCommand<Prefs>('get_prefs');
    var returunLang : Language;
    if (is_first_run) {
        var lang = getLangFormNavigator();
        Prefs.language = lang;
        await invokeCommand<Prefs>('set_prefs', { value: Prefs });
        returunLang = lang;
    }
    else
    {
        returunLang = Prefs.language as Language || 'en';
    }

    InitLang = returunLang;
    return returunLang;
}


export function getLangFormNavigator() : Language
{
    return navigator.language.replace('-', '') as Language;
}

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


