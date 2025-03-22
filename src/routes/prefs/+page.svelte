<script lang="ts">
	import PathPref from '$lib/prefs/PathPref.svelte';
	import LaunchModePref from '$lib/prefs/LaunchModePref.svelte';
	import ZoomLevelPref from '$lib/prefs/ZoomFactorPref.svelte';
	import TogglePref from '$lib/prefs/TogglePref.svelte';
	import ApiKeyPref from '$lib/prefs/ApiKeyPref.svelte';
	import ApiKeyPopup from '$lib/prefs/ApiKeyPopup.svelte';

	import { activeGame } from '$lib/stores';
	import { type Prefs, type GamePrefs, Platform } from '$lib/models';
	import { onMount } from 'svelte';
	import { invokeCommand } from '$lib/invoke';
	import CustomArgsPref from '$lib/prefs/CustomArgsPref.svelte';
	import LargePrefsHeading from '$lib/prefs/LargePrefsHeading.svelte';
	import SmallPrefsHeading from '$lib/prefs/SmallPrefsHeading.svelte';
	import PlatformPref from '$lib/prefs/PlatformPref.svelte';
	import { platform } from '@tauri-apps/plugin-os';
	//import ColorPref from '$lib/prefs/ColorPref.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import { setColor } from '$lib/theme';

	let prefs: Prefs | null = null;
	let gamePrefs: GamePrefs | null = null;

	$: gameSlug = $activeGame?.slug ?? '';
	$: gamePrefs = prefs?.gamePrefs.get(gameSlug) ?? {
		launchMode: { type: 'launcher' },
		dirOverride: null,
		customArgs: null,
		platform: null
	};

	$: platforms = $activeGame?.platforms ?? [];
	$: needsDirectory = !platforms.some(
		(p) =>
			p === Platform.Steam ||
			(platform() === 'windows' && (p === Platform.EpicGames || p === Platform.XboxStore))
	);

	onMount(async () => {
		await refresh();
	});

	function set<T>(update: (value: T, prefs: Prefs) => void) {
		return async (value: T) => {
			if (prefs === null) return;

			update(value, prefs);
			prefs.gamePrefs.set(gameSlug, gamePrefs!);
			try {
				await invokeCommand('set_prefs', { value: prefs });
			} catch (e) {
				await refresh();
				throw e;
			}
		};
	}

	async function refresh() {
		let newPrefs = await invokeCommand<Prefs>('get_prefs');
		newPrefs.gamePrefs = new Map(Object.entries(newPrefs.gamePrefs));
		prefs = newPrefs;
	}
</script>

<div class="mx-auto flex w-full max-w-4xl flex-col gap-1 overflow-y-auto px-6 pt-2 pb-6">
	{#if prefs !== null && gamePrefs !== null}
		<LargePrefsHeading>Global settings</LargePrefsHeading>

		<SmallPrefsHeading>Locations</SmallPrefsHeading>

		<PathPref
			label="Gale data folder"
			type="dir"
			value={prefs.dataDir}
			set={set((value, prefs) => (prefs.dataDir = value))}
		>
			The folder where mods and profiles are stored. Changing this will move the existing data.
		</PathPref>

		<PathPref
			label="Steam executable"
			type="file"
			value={prefs.steamExePath ?? null}
			set={set((value, prefs) => (prefs.steamExePath = value))}
		>
			Path to the Steam executable (steam.exe on Windows). Used for launching games via Steam.
			<br />
			This is <b>not</b> the location of the game's exe. If you want to manually set the game's
			location, use the <b>Override location</b> option further down.
		</PathPref>

		<SmallPrefsHeading>Appearance</SmallPrefsHeading>

		<!--<ColorPref category="primary" fallback="slate" />-->
		<!--<ColorPref category="accent" fallback="green" />-->

		<InputField on:submit={({ detail }) => setColor(detail, 'primary')} />

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
			Whether to automatically fetch mods every 15 minutes. This will ensure the mod list stays
			relatively up-to-date, but can be disabled to save bandwidth.
			<br />
			To manually trigger a fetch, go to <b>File &gt; Fetch mods</b>.
		</TogglePref>

		<TogglePref
			label="Send telemetry"
			value={prefs.sendTelemetry}
			set={set((value, prefs) => (prefs.sendTelemetry = value))}
		>
			Whether to send anonymous usage metrics when the app starts.
		</TogglePref>

		<LargePrefsHeading>
			{$activeGame?.name} settings
		</LargePrefsHeading>

		<SmallPrefsHeading>Locations</SmallPrefsHeading>

		{#if platforms.length > 0}
			<PlatformPref value={gamePrefs.platform} set={set((value) => (gamePrefs.platform = value))} />
		{/if}

		<PathPref
			label={needsDirectory ? 'Location' : 'Override location'}
			type="dir"
			canClear={true}
			value={gamePrefs.dirOverride}
			set={set((value) => (gamePrefs.dirOverride = value))}
		>
			{#if needsDirectory}
				The location of the {$activeGame?.name} folder.
			{:else}
				Overrides the location of the {$activeGame?.name} folder. If unset, Gale will try to find it
				via the specified platform instead.
			{/if}
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
