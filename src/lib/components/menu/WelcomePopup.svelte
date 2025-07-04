<script lang="ts">
	import GameSelect from '$lib/components/menu/GameSelect.svelte';
	import Popup from '$lib/components/ui/Popup.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import PathPref from '$lib/components/prefs/PathPref.svelte';

	import type { Prefs, R2ImportData } from '$lib/types';
	import ImportR2Flow from '$lib/components/import/ImportR2Flow.svelte';
	import Icon from '@iconify/svelte';
	import * as api from '$lib/api';
	import { onMount } from 'svelte';

	type Props = {
		open?: boolean;
	};

	let { open = $bindable(false) }: Props = $props();

	let stage: 'gameSelect' | 'importProfiles' | 'settings' | 'end' = $state('gameSelect');

	let importFlow: ImportR2Flow | null = $state(null);
	let importData: R2ImportData | null | undefined = $state();

	let prefs: Prefs | null = $state(null);

	onMount(async () => {
		if (await api.state.isFirstRun()) {
			open = true;
			prefs = await api.prefs.get();
		}
	});

	async function onSelectGame() {
		try {
			importData = await api.profile.import.getR2modmanInfo(null);
		} catch {
			importData = null;
		}

		stage = importData === null ? 'settings' : 'importProfiles';
	}

	async function importProfiles() {
		if (await importFlow?.doImport()) {
			stage = 'settings';
		}
	}

	function set<T>(update: (value: T, prefs: Prefs) => void) {
		return async (value: T) => {
			if (prefs === null) return;

			update(value, prefs);
			await api.prefs.set(prefs);
		};
	}
</script>

<Popup title="Welcome to Gale!" canClose={stage === 'end'} bind:open>
	<div class="text-primary-300">
		{#if stage === 'gameSelect'}
			To get started, select a game to mod:
			<GameSelect onselect={onSelectGame} />
		{:else if stage === 'importProfiles'}
			<p>You can automatically transfer profiles from another mod manager to Gale.</p>

			<p class="mt-1">
				You can always import profiles later by going to <b>Import &gt; ...from r2modman</b>.
			</p>

			<ImportR2Flow bind:importData bind:this={importFlow} />

			<div class="mt-2 flex gap-1.5">
				<Button color="primary" class="mr-auto" onclick={() => (stage = 'gameSelect')}>Back</Button>
				<Button color="primary" onclick={() => (stage = 'settings')}>Skip</Button>
				<Button color="accent" onclick={importProfiles}>Import</Button>
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
						set={set((value, prefs) => (prefs.dataDir = value as string))}
					>
						The folder where mods and profiles are stored.
					</PathPref>
				{/if}
			</div>

			<div class="mt-3 flex justify-between">
				<Button
					color="primary"
					onclick={() => (stage = importData === null ? 'gameSelect' : 'importProfiles')}
					>Back</Button
				>
				<Button color="accent" onclick={() => (stage = 'end')}>Next</Button>
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
