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

	import { get } from 'svelte/store';
	import { T, t } from '$i18n';

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

<div class="flex flex-col gap-1 py-4 px-6 w-full overflow-y-auto">
	{#if prefs !== null && gamePrefs !== null}
		<div class="text-2xl mt-2 mb-1 font-bold text-slate-100 border-b border-slate-500 pb-1">
			{get(t)["Global settings"]}
		</div>

		<ZoomLevelPref
			value={prefs.zoomFactor}
			set={set((value, prefs) => (prefs.zoomFactor = value))}
		/>

		<ApiKeyPref />

		<TogglePref
			label="{get(t)["Use download cache"]}"
			disableMessage="This will delete all cached mods. Are you sure?"
			value={prefs.enableModCache}
			set={set((value, prefs) => (prefs.enableModCache = value))}
		>
			{@html get(t)["Use download cache description"]}
		</TogglePref>

		<TogglePref
			label="{get(t)["Fetch mods automatically"]}"
			value={prefs.fetchModsAutomatically}
			set={set((value, prefs) => (prefs.fetchModsAutomatically = value))}
		>
			{@html get(t)["Fetch mods automatically description"]}
		</TogglePref>

		<Separator.Root class="h-2" />

		<PathPref
			label="{get(t)["Steam executable"]}"
			type="file"
			value={prefs.steamExePath ?? null}
			set={set((value, prefs) => (prefs.steamExePath = value ?? undefined))}
		>
			{get(t)["Steam executable description"]}
		</PathPref>

		<PathPref
			label="{get(t)["Steam library"]}"
			type="dir"
			value={prefs.steamLibraryDir ?? null}
			set={set((value, prefs) => (prefs.steamLibraryDir = value ?? undefined))}
		>
			{@html get(t)["Steam library description"]}
		</PathPref>

		<Separator.Root class="h-2" />

		<PathPref
			label="{get(t)["Gale data directory short"]}"
			type="dir"
			value={prefs.dataDir}
			set={set((value, prefs) => (prefs.dataDir = value))}
		>
			{get(t)["Gale data directory description"]}
			<br />
			{get(t)["Dir Change will move"]}
		</PathPref>
		<PathPref
			label="{get(t)["Gale cache directory short"]}"
			type="dir"
			value={prefs.cacheDir}
			set={set((value, prefs) => (prefs.cacheDir = value))}
		>
			{get(t)["Gale cache directory description"]}
			<br />
			{get(t)["Dir Change will move"]}
		</PathPref>

		<div class="text-2xl mt-6 mb-1 font-bold text-slate-100 border-b border-slate-500 pb-1">
			{$activeGame?.displayName} {get(t)["settings"]}
		</div>

		<PathPref
			label="{get(t)["Override game directory"]}"
			type="dir"
			canClear={true}
			value={gamePrefs.dirOverride ?? null}
			set={set((value) => (gamePrefs.dirOverride = value ?? undefined))}
		>
			{T(get(t)["Override game directory description"], {"name": $activeGame?.displayName})}
		</PathPref>

		<LaunchModePref
			value={gamePrefs.launchMode}
			set={set((value) => (gamePrefs.launchMode = value))}
		/>
	{/if}
</div>

<ApiKeyPopup />
