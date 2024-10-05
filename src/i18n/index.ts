import en from './en.json';
import zhCN from './zhCN.json';
import Languages from './Languages';
import { invokeCommand } from '$lib/invoke';
import type { Prefs } from '$lib/models';

const translations = {
    en,
    zhCN
};

interface LanguageMap {
    [key: string]: string
}

export type Language = 'en' | 'zhCN';

export const DefaultLanguage = 'en' as Language;
export const LanguageKeys = Object.keys(Languages) as Language[];
const LanguageMaps = new Map<Language, LanguageMap>();

export let initLang : Language;
export let currentLanguage = await getLangFormPrefs();
export let currentTranslationsMap : LanguageMap;
export let defaultTranslationsMap : LanguageMap;

UpdateMap();

function UpdateMap()
{
    LanguageKeys.forEach((key) => {
        var map = translations[key] as LanguageMap;
        LanguageMaps.set(key, map);
        if (key == currentLanguage)
            currentTranslationsMap = map;
        
        if (key == DefaultLanguage)
            defaultTranslationsMap = map;
        }
    )
}


export function getLangName(lang: Language | string) 
{
    return Languages[lang as Language];
}

export function tArray(keys: string, spilt : string = ',') : string
{
    if (!keys) {
        return '';
    }
    return keys.split(spilt).map((key) => t(key)).join(spilt);
}

export async function getLangFormPrefs() : Promise<Language>
{
    var is_first_run =  await invokeCommand<boolean>('is_first_run');
    var Prefs = await invokeCommand<Prefs>('get_prefs');
    var returunLang : Language;
    if (is_first_run) {
        returunLang = getLangFormNavigator();
        setPrefLang(returunLang, Prefs);
    }
    else
    {
        returunLang = Prefs.language as Language || DefaultLanguage;
    }

    return initLang = returunLang;
}

async function setPrefLang(lang: Language, prefs: Prefs | null = null, reload : boolean = false) {
    var savePrefs = prefs ? prefs : await invokeCommand<Prefs>('get_prefs');
    savePrefs.language = lang;
    await invokeCommand<Prefs>('set_prefs', { value: savePrefs });
    if (reload) {
        location.reload();
    }
}


export function getLangFormNavigator() : Language
{
    return navigator.language.replace('-', '') as Language || DefaultLanguage;
}

export function t(key: string) : string
{
    var value = currentTranslationsMap[key] || defaultTranslationsMap[key];
    if (value)
        return value;

    console.warn(`Missing default translation for key: ${key}`);
    return key;
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

export function TB(isEnable: boolean, enableKey: string, disableKey: string, replacements: { [key: string]: any } = {}): string 
{
    return replaceString(t(isEnable ? enableKey : disableKey), replacements);
}

export function TBR(isEnable: boolean, key: string, enableReplacements: { [key: string]: any } = {}, disableReplacements : { [key: string]: any } = {}): string
{
    return replaceString(t(key), isEnable ? enableReplacements : disableReplacements);
}

export function replaceString(str: string, replacements: { [key: string] : any }): string {
    return str.replace(/%(\w+)%/g, (_, key) => replacements[key] || `%${key}%`).replace(/{(\w+)}/g, (_, key) => replacements[key] || `{${key}}`);
}


