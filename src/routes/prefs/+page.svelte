<script lang="ts">
	import PathPref from '$lib/components/prefs/PathPref.svelte';
	import LaunchModePref from '$lib/components/prefs/LaunchModePref.svelte';
	import ZoomLevelPref from '$lib/components/prefs/ZoomFactorPref.svelte';
	import TogglePref from '$lib/components/prefs/TogglePref.svelte';
	import ApiKeyPref from '$lib/components/prefs/ApiKeyPref.svelte';
	import ApiKeyDialog from '$lib/components/dialogs/ApiKeyDialog.svelte';
	import CustomArgsPref from '$lib/components/prefs/CustomArgsPref.svelte';
	import LargeHeading from '$lib/components/prefs/LargeHeading.svelte';
	import SmallHeading from '$lib/components/prefs/SmallHeading.svelte';
	import PlatformPref from '$lib/components/prefs/PlatformPref.svelte';

	import { activeGame } from '$lib/stores.svelte';
	import type { Prefs, GamePrefs, Platform } from '$lib/types';
	import { onMount } from 'svelte';
	import * as api from '$lib/api';

	import { platform } from '@tauri-apps/plugin-os';
	import ColorPref from '$lib/components/prefs/ColorPref.svelte';

	import Label from '$lib/components/ui/Label.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import { getFont, useNativeMenu, setFont } from '$lib/theme';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';

	let prefs: Prefs | null = $state(null);
	let gamePrefs: GamePrefs | null = $state(null);

	let gameSlug = $derived($activeGame?.slug ?? '');

	$effect(() => {
		gamePrefs = prefs?.gamePrefs.get(gameSlug) ?? {
			launchMode: { type: 'launcher' },
			dirOverride: null,
			customArgs: null,
			platform: null
		};
	});

	let platforms = $derived($activeGame?.platforms ?? []);
	let needsDirectory = $derived(
		!platforms.some(
			(p) => p === 'steam' || (platform() === 'windows' && (p === 'epicGames' || p === 'xboxStore'))
		)
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
				await api.prefs.set(prefs);
			} catch (error) {
				await refresh();
				throw error;
			}
		};
	}

	async function refresh() {
		let newPrefs = await api.prefs.get();
		newPrefs.gamePrefs = new Map(Object.entries(newPrefs.gamePrefs));
		prefs = newPrefs;
	}
</script>

<div class="mx-auto flex w-full max-w-4xl flex-col gap-1 overflow-y-auto px-6 pt-2 pb-6">
	{#if prefs !== null && gamePrefs !== null}
		<LargeHeading>Global settings</LargeHeading>

		<SmallHeading>Locations</SmallHeading>

		<PathPref
			label="Gale data folder"
			type="dir"
			value={prefs.dataDir}
			set={set((value, prefs) => (prefs.dataDir = value as string))}
		>
			The folder where mods and profiles are stored. Changing this will move the existing data.
		</PathPref>

		<SmallHeading>Appearance</SmallHeading>

		<ZoomLevelPref
			value={prefs.zoomFactor}
			set={set((value, prefs) => (prefs.zoomFactor = value))}
		/>

		<ColorPref category="primary" />
		<ColorPref category="accent" />

		<div class="flex items-center">
			<Label>Font family</Label>

			<InputField
				value={getFont()}
				onchange={(value) => setFont(value)}
				placeholder="Nunito Sans"
			/>
		</div>

		<div class="my-1 flex items-center">
			<Label>Use native menubar</Label>

			<Checkbox
				checked={$useNativeMenu}
				onCheckedChange={(value) => {
					console.log('checked');
					$useNativeMenu = value;
				}}
			/>
		</div>

		<SmallHeading>Miscellaneous</SmallHeading>

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

		<TogglePref
			label="Pull before launch"
			value={prefs.pullBeforeLaunch}
			set={set((value, prefs) => (prefs.pullBeforeLaunch = value))}
		>
			Whether to pull updates from synced profiles before launching.
		</TogglePref>

		<LargeHeading>
			{$activeGame?.name} settings
		</LargeHeading>

		<SmallHeading>Locations</SmallHeading>

		{#if platforms.length > 0}
			<PlatformPref
				value={gamePrefs.platform}
				set={set((value) => (gamePrefs!.platform = value))}
			/>
		{/if}

		<PathPref
			label={needsDirectory ? 'Location' : 'Override location'}
			type="dir"
			canClear={true}
			value={gamePrefs.dirOverride}
			set={set((value) => (gamePrefs!.dirOverride = value))}
		>
			{#if needsDirectory}
				The location of the {$activeGame?.name} folder.
			{:else}
				Overrides the location of the {$activeGame?.name} folder. If unset, Gale will try to find it
				via the specified platform instead.
			{/if}
		</PathPref>

		<SmallHeading>Launch</SmallHeading>

		<LaunchModePref
			value={gamePrefs.launchMode}
			set={set((value) => (gamePrefs!.launchMode = value))}
		/>

		<CustomArgsPref
			value={gamePrefs.customArgs}
			set={set((value) => (gamePrefs!.customArgs = value))}
		/>
	{/if}
</div>

<ApiKeyDialog />
