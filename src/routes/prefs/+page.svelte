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

	import type { Prefs, GamePrefs, Platform } from '$lib/types';
	import { onMount } from 'svelte';
	import * as api from '$lib/api';

	import { platform } from '@tauri-apps/plugin-os';
	import ColorPref from '$lib/components/prefs/ColorPref.svelte';

	import Label from '$lib/components/ui/Label.svelte';
	import { useNativeMenu } from '$lib/theme';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import FontFamilyPref from '$lib/components/prefs/FontFamilyPref.svelte';
	import LanguagePref from '$lib/components/prefs/LanguagePref.svelte';
	import { m } from '$lib/paraglide/messages';

	let prefs: Prefs | null = $state(null);
	let gamePrefs: GamePrefs | null = $state(null);

	let gameSlug = $derived(games.active?.slug ?? '');

	$effect(() => {
		gamePrefs = prefs?.gamePrefs.get(gameSlug) ?? {
			launchMode: { type: 'launcher' },
			dirOverride: null,
			customArgs: [],
			customArgsEnabled: false,
			platform: null
		};
	});

	let platforms = $derived(games.active?.platforms ?? []);
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
		<LargeHeading>{m.prefs_global_title()}</LargeHeading>

		<SmallHeading>{m.prefs_locations_title()}</SmallHeading>

		<PathPref
			label={m.prefs_locations_dataFolder()}
			type="dir"
			value={prefs.dataDir}
			set={set((value, prefs) => (prefs.dataDir = value as string))}
		>
			{m.prefs_locations_dataFolder_content()}
		</PathPref>

		<SmallHeading>{m.prefs_appearance_title()}</SmallHeading>

		<LanguagePref value={prefs.language} set={set((value, prefs) => { prefs.language = value })} />

		<ColorPref category="primary" default="slate">{m.prefs_appearance_color_primary_content()}</ColorPref>
		<ColorPref category="accent" default="green">{m.prefs_appearance_color_accent_content()}</ColorPref>

		<FontFamilyPref />

		<ZoomLevelPref
			value={prefs.zoomFactor}
			set={set((value, prefs) => (prefs.zoomFactor = value))}
		/>

		<div class="my-1 flex items-center">
			<Label>{m.prefs_appearance_nativeMenubar_title()}</Label>

			<Checkbox
				checked={useNativeMenu.current}
				onCheckedChange={(value) => {
					useNativeMenu.current = value;
				}}
			/>
		</div>

		<SmallHeading>{m.prefs_miscellaneous_title()}</SmallHeading>

		<ApiKeyPref />

		<TogglePref
			label={m.prefs_miscellaneous_fetchMods_title()}
			value={prefs.fetchModsAutomatically}
			set={set((value, prefs) => (prefs.fetchModsAutomatically = value))}
		>
			{m.prefs_miscellaneous_fetchMods_content_1()}
			<br />
			{m.prefs_miscellaneous_fetchMods_content_2()}<b>{m.prefs_miscellaneous_fetchMods_content_3()}</b>.
		</TogglePref>
		<TogglePref
			label={m.prefs_miscellaneous_pullBeforeLaunch_title()}
			value={prefs.pullBeforeLaunch}
			set={set((value, prefs) => (prefs.pullBeforeLaunch = value))}
		>
			{m.prefs_miscellaneous_pullBeforeLaunch_content()}
		</TogglePref>

		<LargeHeading>
			{m.prefs_gameSettings_title({ game : games.active?.name ?? m.unknown() })}
		</LargeHeading>

		<SmallHeading>{m.prefs_gameSettings_locations_title()}</SmallHeading>

		{#if platforms.length > 0}
			<PlatformPref
				value={gamePrefs.platform}
				set={set((value) => (gamePrefs!.platform = value))}
			/>
		{/if}

		<PathPref
			label={m[`prefs_gameSettings_locations_dirOverride_title${needsDirectory ? '_needs' : ''}`]()}
			type="dir"
			canClear={true}
			value={gamePrefs.dirOverride}
			set={set((value) => (gamePrefs!.dirOverride = value))}
		>
		{m[`prefs_gameSettings_locations_dirOverride_content${needsDirectory ? '_needs' : ''}`]({ game: games.active?.name ?? m.unknown() })}
		</PathPref>

		<SmallHeading>{m.prefs_gameSettings_launch_title()}</SmallHeading>

		<LaunchModePref
			platform={gamePrefs.platform ?? games.active?.platforms[0] ?? m.unknown()}
			value={gamePrefs.launchMode}
			set={set((value) => (gamePrefs!.launchMode = value))}
		/>

		<CustomArgsPref
			value={gamePrefs.customArgs}
			enabled={gamePrefs.customArgsEnabled}
			setValue={set((value) => (gamePrefs!.customArgs = value))}
			setEnabled={set((value) => (gamePrefs!.customArgsEnabled = value))}
		/>

		{#if profiles.active}
			<LargeHeading>Profile settings</LargeHeading>

			<SmallHeading>Launch</SmallHeading>

			<CustomArgsPref
				value={profiles.active.customArgs}
				enabled={profiles.active.customArgsEnabled}
				setValue={async (value) =>
					await api.profile.setCustomArgs(value, profiles.active!.customArgsEnabled)}
				setEnabled={async (value) =>
					await api.profile.setCustomArgs(profiles.active!.customArgs, value)}
			/>
		{/if}
	{/if}
</div>

<ApiKeyDialog />
