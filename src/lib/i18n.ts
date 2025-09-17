import { m } from '$lib/paraglide/messages';
import { getLocale, locales, setLocale, type Locale } from '$lib/paraglide/runtime';
import { toSentenceCase as toSentenceCaseLatin } from 'js-convert-case';
import * as api from '$lib/api';
import { locale } from '@tauri-apps/plugin-os';

export function setLanguage(lang: Locale | string) {
	setLocale(lang as Locale);
	console.log(`Language changed to ${lang}`);
}

export async function refreshLanguage() {
	let lang: string;
	let prefs = await api.prefs.get();

	if (await api.state.isFirstRun()) {
		let systemLocale = await locale();
		if (!systemLocale || !locales.includes(systemLocale as Locale)) {
			return;
		}

		lang = systemLocale;

		prefs.language = lang;
		await api.prefs.set(prefs);
	} else {
		lang = prefs.language;
	}

	if (lang !== getLocale()) {
		setLanguage(lang);
	}
}

export const languageTitle: Record<Locale, string> = locales.reduce(
	(acc, item) => {
		acc[item] = m.language_name({}, { locale: item });
		return acc;
	},
	{} as Record<Locale, string>
);

export function isLatinAlphabet(str: string): boolean {
	return /^[a-zA-Z\s]*$/.test(str);
}

export function pluralizeOption(
	isPlural: boolean | number,
	origin: string,
	singular: string,
	plural: string
): string {
	if (typeof isPlural === 'number' && isPlural !== 1) {
		return origin;
	}

	if (typeof isPlural === 'boolean' && !isPlural) {
		return origin;
	}

	if (!isLatinAlphabet(origin)) {
		return origin;
	}

	return origin.replace(new RegExp(singular, 'g'), plural);
}

export function toSentenceCase(str: string): string {
	return isLatinAlphabet(str) ? toSentenceCaseLatin(str) : str;
}
