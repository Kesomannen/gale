<script lang="ts">
	import PathPref from '$lib/prefs/PathPref.svelte';
	import LaunchModePref from '$lib/prefs/LaunchModePref.svelte';
	import ZoomLevelPref from '$lib/prefs/ZoomFactorPref.svelte';
	import TogglePref from '$lib/prefs/TogglePref.svelte';
	import ApiKeyPref from '$lib/prefs/ApiKeyPref.svelte';
	import ApiKeyPopup from '$lib/prefs/ApiKeyPopup.svelte';

	import { activeGame } from '$lib/stores';
	import { Separator } from 'bits-ui';
	import type { Prefs, GamePrefs } from '$lib/models';
	import { onMount } from 'svelte';
	import { invokeCommand } from '$lib/invoke';
	import { T, t } from '$i18n';
	import LanguagePref from '$lib/prefs/LanguagePref.svelte';

	let prefs: Prefs | null = null;
	let gamePrefs: GamePrefs | null = null;

	$: gameId = $activeGame?.id ?? '';
	$: gamePrefs = prefs?.gamePrefs.get(gameId) ?? {
		launchMode: { type: 'steam' },
		dirOverride: undefined
	};

	onMount(async () => {
		let newPrefs = await invokeCommand<Prefs>('get_prefs');
		newPrefs.gamePrefs = new Map(Object.entries(newPrefs.gamePrefs));
		prefs = newPrefs;
	});

	function set<T>(update: (value: T, prefs: Prefs) => void) {
		return async (value: T) => {
			if (prefs === null) return;

			update(value, prefs);
			prefs.gamePrefs.set(gameId, gamePrefs!);
			await invokeCommand('set_prefs', { value: prefs });
		};
	}
</script>

<div class="flex w-full flex-col gap-1 overflow-y-auto px-6 py-4">
	{#if prefs !== null && gamePrefs !== null}
		<div class="mb-1 mt-2 border-b border-slate-500 pb-1 text-2xl font-bold text-slate-100">
			{t("Global settings")}
		</div>

		<ZoomLevelPref
			value={prefs.zoomFactor}
			set={set((value, prefs) => (prefs.zoomFactor = value))}
		/>

		<ApiKeyPref />
		<LanguagePref set={set((value, prefs) => (prefs.language = value))}/>

		<TogglePref
			label={t("Fetch mods automatically")}
			value={prefs.fetchModsAutomatically}
			set={set((value, prefs) => (prefs.fetchModsAutomatically = value))}
		>
			{@html t("Fetch mods automatically description")}
		</TogglePref>


		<PathPref
			label="{t("Steam executable")}"
			type="file"
			value={prefs.steamExePath ?? null}
			set={set((value, prefs) => (prefs.steamExePath = value ?? undefined))}
		>
			{t("Steam executable description")}
		</PathPref>

		<PathPref
			label="{t("Steam library")}"
			type="dir"
			value={prefs.steamLibraryDir ?? null}
			set={set((value, prefs) => (prefs.steamLibraryDir = value ?? undefined))}
		>
			{@html t("Steam library description")}
		</PathPref>

		<PathPref
			label="{t("Gale data directory short")}"
			type="dir"
			value={prefs.dataDir}
			set={set((value, prefs) => (prefs.dataDir = value ?? prefs.dataDir))}
		>
			{t("Gale data directory description")}
			<br />
			{t("Dir Change will move")}
		</PathPref>

		<div class="mb-1 mt-6 border-b border-slate-500 pb-1 text-2xl font-bold text-slate-100">
			{$activeGame?.displayName} {t("settings")}
		</div>

		<PathPref
			label="{t("Override game directory")}"
			type="dir"
			canClear={true}
			value={gamePrefs.dirOverride ?? null}
			set={set((value) => (gamePrefs.dirOverride = value ?? undefined))}
		>
			{T("Override game directory description", {"name": $activeGame?.displayName})}
		</PathPref>

		<LaunchModePref
			value={gamePrefs.launchMode}
			set={set((value) => (gamePrefs.launchMode = value))}
		/>
	{/if}
</div>

<ApiKeyPopup />
