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
			Global settings
		</div>

		<ZoomLevelPref
			value={prefs.zoomFactor}
			set={set((value, prefs) => (prefs.zoomFactor = value))}
		/>

		<ApiKeyPref />

		<TogglePref
			label="Use download cache"
			disableMessage="This will delete all cached mods. Are you sure?"
			value={prefs.enableModCache}
			set={set((value, prefs) => (prefs.enableModCache = value))}
		>
			Whether to cache downloaded mods. This speeds up install times and lowers bandwidth usage, but
			can take a considerable amount of disk space.
			<br />
			<b>Warning:</b> Disabling this will delete the existing cache.
		</TogglePref>

		<Separator.Root class="h-2" />

		<PathPref
			label="Steam executable"
			type="file"
			value={prefs.steamExePath ?? null}
			set={set((value, prefs) => (prefs.steamExePath = value ?? undefined))}
		>
			Path to the Steam executable.
		</PathPref>

		<PathPref
			label="Steam library"
			type="dir"
			value={prefs.steamLibraryDir ?? null}
			set={set((value, prefs) => (prefs.steamLibraryDir = value ?? undefined))}
		>
			Path to the Steam game library. This should <b>contain</b> the 'steamapps' directory.
		</PathPref>

		<Separator.Root class="h-2" />

		<PathPref
			label="Data directory"
			type="dir"
			value={prefs.dataDir}
			set={set((value, prefs) => (prefs.dataDir = value))}
		>
			Directory where profiles and other app data is stored.
			<br />
			Changing this will move the existing data.
		</PathPref>

		<PathPref
			label="Temp directory"
			type="dir"
			value={prefs.tempDir}
			set={set((value, prefs) => (prefs.tempDir = value))}
		>
			Directory where temporary files are stored, for example import and export files.
		</PathPref>

		<PathPref
			label="Download cache directory"
			type="dir"
			value={prefs.cacheDir}
			set={set((value, prefs) => (prefs.cacheDir = value))}
		>
			Directory where cached mods are stored.
			<br />
			Changing this will move the existing cache.
		</PathPref>

		<div class="text-2xl mt-6 mb-1 font-bold text-slate-100 border-b border-slate-500 pb-1">
			{$activeGame?.displayName} settings
		</div>

		<PathPref
			label="Override game directory"
			type="dir"
			canClear={true}
			value={gamePrefs.dirOverride ?? null}
			set={set((value) => (gamePrefs.dirOverride = value ?? undefined))}
		>
			Path to the {$activeGame?.displayName} game directory. Leave empty to use the default Steam library.
		</PathPref>

		<LaunchModePref
			value={gamePrefs.launchMode}
			set={set((value) => (gamePrefs.launchMode = value))}
		/>
	{/if}
</div>

<ApiKeyPopup />
