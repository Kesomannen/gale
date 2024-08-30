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
			label="Fetch mods automatically"
			value={prefs.fetchModsAutomatically}
			set={set((value, prefs) => (prefs.fetchModsAutomatically = value))}
		>
			Whether to automatically start fetching mods when a game is selected and every 15 minutes
			thereafter. This will ensure the mod list is up-to-date, but can be disabled to save
			bandwidth.
			<br />
			To manually trigger a fetch, go to <b>File > Fetch mods</b>.
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

		<PathPref
			label="Data directory"
			type="dir"
			value={prefs.dataDir}
			set={set((value, prefs) => (prefs.dataDir = value))}
		>
			Directory where mods and profiles are stored.
			<br />
			Changing this will move the existing data.
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
