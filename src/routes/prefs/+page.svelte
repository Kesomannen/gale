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
	import CustomArgsPref from '$lib/prefs/CustomArgsPref.svelte';
	import AccentColorPref from '$lib/prefs/AccentColorPref.svelte';
	import LargePrefsHeading from '$lib/prefs/LargePrefsHeading.svelte';
	import SmallPrefsHeading from '$lib/prefs/SmallPrefsHeading.svelte';

	let prefs: Prefs | null = null;
	let gamePrefs: GamePrefs | null = null;

	$: gameId = $activeGame?.slug ?? '';
	$: gamePrefs = prefs?.gamePrefs.get(gameId) ?? {
		launchMode: { type: 'steam' },
		dirOverride: null,
		customArgs: null
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

<div class="flex w-full flex-col gap-1 overflow-y-auto px-6 pb-6 pt-2">
	{#if prefs !== null && gamePrefs !== null}
		<LargePrefsHeading>Global settings</LargePrefsHeading>

		<SmallPrefsHeading>Locations</SmallPrefsHeading>

		<PathPref
			label="Steam executable"
			type="file"
			value={prefs.steamExePath ?? null}
			set={set((value, prefs) => (prefs.steamExePath = value))}
		>
			Path to the Steam executable.
		</PathPref>

		<PathPref
			label="Steam library"
			type="dir"
			value={prefs.steamLibraryDir ?? null}
			set={set((value, prefs) => (prefs.steamLibraryDir = value))}
		>
			Path to your default Steam game library.
		</PathPref>

		<PathPref
			label="Data directory"
			type="dir"
			value={prefs.dataDir}
			set={set((value, prefs) => (prefs.dataDir = value))}
		>
			Directory where mods and profiles are stored. Changing this will move the existing data.
		</PathPref>

		<SmallPrefsHeading>Appearance</SmallPrefsHeading>

		<AccentColorPref />

		<ZoomLevelPref
			value={prefs.zoomFactor}
			set={set((value, prefs) => (prefs.zoomFactor = value))}
		/>

		<SmallPrefsHeading>Miscellaneous</SmallPrefsHeading>

		<ApiKeyPref />

		<TogglePref
			label="Fetch mods automatically"
			value={prefs.fetchModsAutomatically}
			set={set((value, prefs) => (prefs.fetchModsAutomatically = value))}
		>
			Whether to automatically fetch mods every 15 minutes. This will ensure the mod list is
			up-to-date, but can be disabled to save bandwidth.
			<br />
			To manually trigger a fetch, go to <b>File &gt; Fetch mods</b>.
		</TogglePref>

		<LargePrefsHeading>
			{$activeGame?.name} settings
		</LargePrefsHeading>

		<SmallPrefsHeading>Locations</SmallPrefsHeading>

		<PathPref
			label="Override directory"
			type="dir"
			canClear={true}
			value={gamePrefs.dirOverride ?? null}
			set={set((value) => (gamePrefs.dirOverride = value))}
		>
			Path to the {$activeGame?.name} game directory. Leave empty to use the default Steam library.
		</PathPref>

		<SmallPrefsHeading>Launch</SmallPrefsHeading>

		<LaunchModePref
			value={gamePrefs.launchMode}
			set={set((value) => (gamePrefs.launchMode = value))}
		/>

		<CustomArgsPref
			value={gamePrefs.customArgs}
			set={set((value) => (gamePrefs.customArgs = value))}
		/>
	{/if}
</div>

<ApiKeyPopup />
