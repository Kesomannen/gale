import { invoke } from '$lib/invoke';
import type { TranslateRequest, TranslateResponse, TranslationPrefs } from '$lib/types';

export const translateMods = (mods: TranslateRequest[], targetLanguage: string) =>
	invoke<TranslateResponse[]>('translate_mods', { mods, targetLanguage });

export const getPrefs = () => invoke<TranslationPrefs>('get_translation_prefs');

export const setPrefs = (prefs: TranslationPrefs) =>
	invoke('set_translation_prefs', { prefs });
