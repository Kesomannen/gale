<script lang="ts">
	import GameSelection from '$lib/components/GameSelection.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import BigButton from '$lib/components/BigButton.svelte';
	import PathPref from '$lib/prefs/PathPref.svelte';

	import type { Prefs, R2ImportData } from '$lib/models';

	import { invokeCommand } from '$lib/invoke';
	import { onMount } from 'svelte';
	import ImportR2Flow from '$lib/import/ImportR2Flow.svelte';
	import Icon from '@iconify/svelte';

	import { T, t } from '$i18n';

	export let open = false;

	let stage: 'gameSelect' | 'importProfiles' | 'settings' | 'end' = 'gameSelect';

	let importFrom: 'r2modman' | 'thunderstore' = 'r2modman';
	let importData: R2ImportData = {
		r2modman: undefined,
		thunderstore: undefined
	};

	let importFlow: ImportR2Flow;

	let prefs: Prefs | null = null;

	$: importText =
		importData.r2modman && importData.thunderstore
			? `r2modman ${t["or"]} Thunderstore Mod Manager`
			: importData.r2modman
				? 'r2modman'
				: 'Thunderstore Mod Manager';

	onMount(async () => {
		if (await invokeCommand<boolean>('is_first_run')) {
			open = true;
			prefs = await invokeCommand('get_prefs');
		}
	});

	async function onSelectGame() {
		let result = await invokeCommand<R2ImportData>('get_r2modman_info');

		if (!result.r2modman && !result.thunderstore) {
			stage = 'settings';
			return;
		}

		importData = result;

		if (result.r2modman) {
			result.r2modman.include = result.r2modman.profiles.map(() => true);
			importFrom = 'r2modman';
		}

		if (result.thunderstore) {
			result.thunderstore.include = result.thunderstore.profiles.map(() => true);
			importFrom = 'thunderstore';
		}

		stage = 'importProfiles';
	}

	async function importProfiles() {
		await importFlow.doImport();
		stage = 'settings';
	}

	function set<T>(update: (value: T, prefs: Prefs) => void) {
		return async (value: T) => {
			if (prefs === null) return;

			update(value, prefs);
			await invokeCommand('set_prefs', { value: prefs });
		};
	}
</script>

<Popup title="{t['Welcome to Gale']}" canClose={stage === 'end'} bind:open maxWidth="[55%]">
	<div class="text-slate-300">
		{#if stage === 'gameSelect'}
			{t['Welcome to Gale description 1']}
			<GameSelection onSelect={onSelectGame} />
		{:else if stage === 'importProfiles' && importData}
			<p>
				{T(t['Welcome to Gale description 2'], {"importText": importText})}
			</p>

			<p class="mt-1">
				{t['Welcome to Gale description 3']}
			</p>

			<p class="mt-1">
				{@html t['Welcome to Gale description 4']}
			</p>

			<ImportR2Flow bind:importData bind:importFrom bind:this={importFlow} />

			<div class="flex mt-2 gap-1.5">
				<BigButton color="gray" on:click={() => (stage = 'gameSelect')}>{t["Back"]}</BigButton>
				<div class="flex-grow" />
				<BigButton color="gray" on:click={() => (stage = 'settings')}>{t["Skip"]}</BigButton>
				<BigButton color="green" on:click={importProfiles}>Import</BigButton>
			</div>
		{:else if stage === 'settings'}
			<p>{t['Welcome to Gale description 5']}</p>

			<p class="mt-1">
				{t['Welcome to Gale description 6']} <Icon icon="mdi:settings" class="inline mb-1" />
				<b>{t['Settings']}</b>
				{t['Welcome to Gale description 7']}
			</p>

			<div class="flex flex-col mt-3 gap-1">
				{#if prefs !== null}
					<PathPref
						label="{t["Steam executable"]}"
						type="file"
						value={prefs.steamExePath ?? null}
						set={set((value, prefs) => (prefs.steamExePath = value ?? undefined))}
					>
						{t["Steam executable description"]}
					</PathPref>

					<PathPref
						label="{t["Steam library"]}"
						type="dir"
						value={prefs.steamLibraryDir ?? null}
						set={set((value, prefs) => (prefs.steamLibraryDir = value ?? undefined))}
					>
						{t["Steam library description"]}
					</PathPref>

					<PathPref
						label="{t["Gale data directory"]}"
						type="dir"
						value={prefs.dataDir}
						set={set((value, prefs) => (prefs.dataDir = value))}
					>
						{t["Gale data directory description"]}
					</PathPref>

					<PathPref
						label="{t["Gale cache directory"]}"
						type="dir"
						value={prefs.cacheDir}
						set={set((value, prefs) => (prefs.cacheDir = value))}
					>
						{t["Gale cache directory description"]}
					</PathPref>
				{/if}
			</div>

			<div class="flex mt-3 justify-between">
				<BigButton
					color="gray"
					on:click={() =>
						(stage =
							importData.r2modman || importData.thunderstore ? 'importProfiles' : 'gameSelect')}
					>{t["Back"]}</BigButton
				>
				<BigButton color="green" on:click={() => (stage = 'end')}>{t["Next"]}</BigButton>
			</div>
		{:else if stage === 'end'}
			<p>{t['Welcome to Gale description 8']}</p>

			<p class="mt-1">
				{t['Welcome to Gale description 9']} 
				<a
					href="https://discord.gg/lcmod"
					target="_blank"
					class="text-green-400 hover:underline">Lethal Company Modding Discord server</a
				>
				{t['Welcome to Gale description 10']}
			</p>
		{/if}
	</div>
</Popup>
