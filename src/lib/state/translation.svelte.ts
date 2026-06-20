import * as api from '$lib/api';
import type { Mod, TranslateRequest, TranslateResponse, TranslationPrefs } from '$lib/types';
import { getLocale } from '$lib/paraglide/runtime';

class TranslationState {
	prefs: TranslationPrefs | null = $state(null);
	cache: Record<string, TranslateResponse> = $state({});
	nameLookup: Record<string, string> = $state({});
	configCache: Record<string, TranslateResponse> = $state({});
	translating: Record<string, boolean> = $state({});
	error: string | null = $state(null);
	translateRequest: number = $state(0);
	showTranslation: boolean = $state(true);

	constructor() {
		this.loadPrefs();
	}

	async loadPrefs() {
		try {
			this.prefs = await api.translation.getPrefs();
		} catch (err) {
			console.error('Failed to load translation prefs:', err);
			this.prefs = { enabled: false, apiUrl: '', apiKey: '', model: 'gpt-4o-mini', batchSize: 20 };
		}
	}

	async savePrefs() {
		if (!this.prefs) return;
		try {
			await api.translation.setPrefs(this.prefs);
		} catch (err) {
			console.error('Failed to save translation prefs:', err);
		}
	}

	requestTranslateVisible() {
		this.translateRequest++;
	}

	getTargetLanguage(): string {
		const locale = getLocale();
		const languageMap: Record<string, string> = {
			en: 'English',
			zh: 'Simplified Chinese',
			'zh-CN': 'Simplified Chinese',
			'zh-TW': 'Traditional Chinese',
			ja: 'Japanese',
			ko: 'Korean',
			es: 'Spanish',
			fr: 'French',
			de: 'German',
			ru: 'Russian',
			pt: 'Portuguese',
			'pt-BR': 'Brazilian Portuguese',
			it: 'Italian',
			nl: 'Dutch',
			pl: 'Polish',
			tr: 'Turkish',
			th: 'Thai',
			vi: 'Vietnamese'
		};
		const normalized = locale.split('-')[0].toLowerCase();
		return languageMap[locale] || languageMap[normalized] || 'English';
	}

	isTranslating(key: string): boolean {
		return !!this.translating[key];
	}

	getTranslation(uuid: string): TranslateResponse | null {
		return this.cache[uuid] || null;
	}

	getDisplayName(uuid: string, originalName: string): string {
		if (!this.showTranslation) return originalName;
		if (uuid) {
			const t = this.cache[uuid];
			if (t) return t.name;
		}
		const cached = this.cache[`_name_${originalName}`];
		if (cached) return cached.name;
		return originalName;
	}

	getDisplayDescription(uuid: string, originalDesc: string | null): string | null {
		if (!this.showTranslation) return originalDesc;
		if (!uuid) return originalDesc;
		const t = this.cache[uuid];
		return t?.description ?? originalDesc;
	}

	async translateMod(mod: Mod): Promise<TranslateResponse | null> {
		if (!this.prefs?.apiUrl || !this.prefs?.apiKey) return null;
		if (this.cache[mod.uuid]) return this.cache[mod.uuid];
		if (this.translating[mod.uuid]) return null;

		this.translating = { ...this.translating, [mod.uuid]: true };
		this.error = null;

		try {
			const request: TranslateRequest = { uuid: mod.uuid, name: mod.name, description: mod.description };
			const results = await api.translation.translateMods([request], this.getTargetLanguage());
			if (results.length > 0) {
				const result = results[0];
				this.cache = { ...this.cache, [mod.uuid]: result };
				this.nameLookup = { ...this.nameLookup, [mod.name.toLowerCase()]: mod.uuid };
				return result;
			}
		} catch (err) {
			console.error('Translation failed:', err);
			this.error = err instanceof Error ? err.message : 'Translation failed';
		} finally {
			const { [mod.uuid]: _, ...rest } = this.translating;
			this.translating = rest;
		}
		return null;
	}

	async translateMods(mods: Mod[]): Promise<void> {
		if (!this.prefs?.apiUrl || !this.prefs?.apiKey) return;

		const toTranslate = mods.filter((mod) => !this.cache[mod.uuid] && !this.translating[mod.uuid]);
		if (toTranslate.length === 0) return;

		this.error = null;
		const newTranslating = { ...this.translating };
		toTranslate.forEach((m) => (newTranslating[m.uuid] = true));
		this.translating = newTranslating;

		try {
			const requests: TranslateRequest[] = toTranslate.map((mod) => ({
				uuid: mod.uuid, name: mod.name, description: mod.description
			}));
			const results = await api.translation.translateMods(requests, this.getTargetLanguage());

			const newCache = { ...this.cache };
			const newNameLookup = { ...this.nameLookup };
			toTranslate.forEach((mod, i) => {
				if (i < results.length) {
					newCache[mod.uuid] = results[i];
					newNameLookup[mod.name.toLowerCase()] = mod.uuid;
				}
			});
			this.cache = newCache;
			this.nameLookup = newNameLookup;
		} catch (err) {
			console.error('Translation failed:', err);
			this.error = err instanceof Error ? err.message : 'Translation failed';
		} finally {
			const newTranslating = { ...this.translating };
			toTranslate.forEach((m) => delete newTranslating[m.uuid]);
			this.translating = newTranslating;
		}
	}

	async translateNames(names: string[]): Promise<void> {
		await this.translateGenericBatch(
			names.map((name) => ({ key: `_name_${name}`, name, description: null as string | null })),
			this.cache,
			(newEntries) => { this.cache = { ...this.cache, ...newEntries }; }
		);
	}

	async testConnection(): Promise<{ ok: boolean; message: string }> {
		if (!this.prefs?.apiUrl || !this.prefs?.apiKey) {
			return { ok: false, message: 'Please fill in API URL and API Key' };
		}
		try {
			const request: TranslateRequest = { uuid: 'test', name: 'Test Mod', description: 'A test mod for verification' };
			const results = await api.translation.translateMods([request], 'Simplified Chinese');
			if (results.length > 0) {
				return { ok: true, message: `Success! Translated to: ${results[0].name}` };
			}
			return { ok: false, message: 'No response from API' };
		} catch (err) {
			return { ok: false, message: err instanceof Error ? err.message : 'Connection failed' };
		}
	}

	clearCache(): void {
		this.cache = {};
		this.nameLookup = {};
		this.configCache = {};
		this.translating = {};
		this.error = null;
	}

	getConfigTranslation(filePath: string, entryName: string): TranslateResponse | null {
		return this.configCache[`${filePath}::${entryName}`] || null;
	}

	async translateConfigEntries(
		filePath: string,
		entries: { name: string; description: string | null }[]
	): Promise<void> {
		await this.translateGenericBatch(
			entries.map((e) => ({ key: `${filePath}::${e.name}`, name: e.name, description: e.description })),
			this.configCache,
			(newEntries) => { this.configCache = { ...this.configCache, ...newEntries }; }
		);
	}

	private async translateGenericBatch(
		items: { key: string; name: string; description: string | null }[],
		targetCache: Record<string, TranslateResponse>,
		apply: (entries: Record<string, TranslateResponse>) => void
	): Promise<void> {
		if (!this.prefs?.apiUrl || !this.prefs?.apiKey || items.length === 0) return;

		const toTranslate = items.filter((item) => !targetCache[item.key]);
		if (toTranslate.length === 0) return;

		try {
			const requests: TranslateRequest[] = toTranslate.map((item) => ({
				uuid: item.key, name: item.name, description: item.description
			}));
			const results = await api.translation.translateMods(requests, this.getTargetLanguage());

			const newEntries: Record<string, TranslateResponse> = {};
			toTranslate.forEach((item, i) => {
				if (i < results.length) {
					newEntries[item.key] = results[i];
				}
			});
			apply(newEntries);
		} catch (err) {
			console.error('Translation failed:', err);
		}
	}
}

const translation = new TranslationState();

export default translation;
