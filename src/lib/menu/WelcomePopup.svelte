<script lang="ts">
	import GameSelection from '$lib/menu/GameSelection.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import BigButton from '$lib/components/Button.svelte';
	import PathPref from '$lib/prefs/PathPref.svelte';

	import type { Prefs, R2ImportData } from '$lib/models';

	import { invokeCommand } from '$lib/invoke';
	import { onMount } from 'svelte';
	import ImportR2Flow from '$lib/import/ImportR2Flow.svelte';
	import Icon from '@iconify/svelte';
	import { invoke } from '@tauri-apps/api/core';

	type Props = {
		open?: boolean;
	};

	let { open = $bindable(false) }: Props = $props();

	let stage: 'gameSelect' | 'importProfiles' | 'settings' | 'end' = $state('gameSelect');

	let importFlow: ImportR2Flow = $state();
	let importData: R2ImportData | null | undefined = $state();

	let prefs: Prefs | null = $state(null);

	onMount(async () => {
		if (await invokeCommand<boolean>('is_first_run')) {
			open = true;
			prefs = await invokeCommand('get_prefs');
		}
	});

	async function onSelectGame() {
		try {
			importData = await invoke('get_r2modman_info');
		} catch {
			importData = null;
		}

		stage = importData === null ? 'settings' : 'importProfiles';
	}

	async function importProfiles() {
		if (await importFlow.doImport()) {
			stage = 'settings';
		}
	}

	function set<T>(update: (value: T, prefs: Prefs) => void) {
		return async (value: T) => {
			if (prefs === null) return;

			update(value, prefs);
			await invokeCommand('set_prefs', { value: prefs });
		};
	}
</script>

<Popup title="Welcome to Gale!" canClose={stage === 'end'} bind:open>
	<div class="text-primary-300">
		{#if stage === 'gameSelect'}
			To get started, select a game to mod:
			<GameSelection onSelect={onSelectGame} />
		{:else if stage === 'importProfiles'}
			<p>You can automatically transfer profiles from another mod manager to Gale.</p>

			<p class="mt-1">
				You can always import profiles later by going to <b>Import &gt; ...from r2modman</b>.
			</p>

			<ImportR2Flow bind:importData bind:this={importFlow} />

			<div class="mt-2 flex gap-1.5">
				<BigButton color="primary" class="mr-auto" on:click={() => (stage = 'gameSelect')}
					>Back</BigButton
				>
				<BigButton color="primary" on:click={() => (stage = 'settings')}>Skip</BigButton>
				<BigButton color="accent" on:click={importProfiles}>Import</BigButton>
			</div>
		{:else if stage === 'settings'}
			<p>
				Let's make sure your settings are to your liking.
				<br />
				You can always edit these later by going to <Icon icon="mdi:settings" class="mb-1 inline" />
				<b>Settings</b>.
			</p>

			<div class="mt-3 flex flex-col gap-1">
				{#if prefs !== null}
					<PathPref
						label="Gale data folder"
						type="dir"
						value={prefs.dataDir}
						set={set((value, prefs) => (prefs.dataDir = value))}
					>
						The folder where mods and profiles are stored.
					</PathPref>
				{/if}
			</div>

			<div class="mt-3 flex justify-between">
				<BigButton
					color="primary"
					on:click={() => (stage = importData === null ? 'gameSelect' : 'importProfiles')}
					>Back</BigButton
				>
				<BigButton color="accent" on:click={() => (stage = 'end')}>Next</BigButton>
			</div>
		{:else if stage === 'end'}
			<p>That's it, you're all set up to start modding!</p>

			<p class="mt-1">
				If you have any questions or need help, feel free to ask in the <a
					href="https://discord.gg/sfuWXRfeTt"
					target="_blank"
					class="text-accent-400 hover:underline">Discord server</a
				>.
			</p>
		{/if}
	</div>
</Popup>
