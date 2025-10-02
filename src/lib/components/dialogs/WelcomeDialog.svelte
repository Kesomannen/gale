<script lang="ts">
	import GameSelect from '$lib/components/toolbar/GameSelect.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import PathPref from '$lib/components/prefs/PathPref.svelte';

	import type { Prefs, R2ImportData } from '$lib/types';
	import ImportR2Flow from '$lib/components/ui/ImportR2Flow.svelte';
	import Icon from '@iconify/svelte';
	import * as api from '$lib/api';
	import { onMount } from 'svelte';
	import ColorPref from '../prefs/ColorPref.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { m } from '$lib/paraglide/messages';

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
			importData = await invoke<R2ImportData | null>('get_r2modman_info');
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

<Dialog title={m.welcomeDialog_title()} canClose={stage === 'end'} bind:open>
	<div class="text-primary-300">
		{#if stage === 'gameSelect'}
			{m.welcomeDialog_content_gameSelect()}
			<GameSelect onselect={onSelectGame} />
		{:else if stage === 'importProfiles'}
			<p>{m.welcomeDialog_content_importProfiles_1()}</p>

			<p class="mt-1">
				{m.welcomeDialog_content_importProfiles_2()}<b>{m.welcomeDialog_content_importProfiles_3()}</b>.
			</p>

			<ImportR2Flow bind:importData bind:this={importFlow} />

			<div class="mt-2 flex gap-1.5">
				<Button color="primary" class="mr-auto" onclick={() => (stage = 'gameSelect')}>{m.welcomeDialog_button_back()}</Button>
				<Button color="primary" onclick={() => (stage = 'settings')}>{m.welcomeDialog_button_skip()}</Button>
				<Button color="accent" onclick={importProfiles}>{m.welcomeDialog_button_import()}</Button>
			</div>
		{:else if stage === 'settings'}
			<p>
				{m.welcomeDialog_content_settings_1()}
				<br />
				{m.welcomeDialog_content_settings_2()}<Icon icon="mdi:settings" class="mb-1 inline" />
				<b>{m.welcomeDialog_content_settings_3()}</b>.
			</p>

			<div class="mt-3 flex flex-col gap-1">
				{#if prefs}
					<PathPref
						label={m.welcomeDialog_settings_path_title()}
						type="dir"
						value={prefs.dataDir}
						set={set((value, prefs) => (prefs.dataDir = value as string))}
					>
						{m.welcomeDialog_settings_path_content()}
					</PathPref>

					<ColorPref category="primary" default="slate">{m.welcomeDialog_content_color_primary()}</ColorPref>
					<ColorPref category="accent" default="green">{m.welcomeDialog_content_color_accent()}</ColorPref>
				{/if}
			</div>

			<div class="mt-3 flex justify-between">
				<Button
				color="primary"
				onclick={() => (stage = importData === null ? 'gameSelect' : 'importProfiles')}
				>
				{m.welcomeDialog_button_back()}
				</Button>
				<Button color="accent" onclick={() => (stage = 'end')}>{m.welcomeDialog_button_next()}</Button>
			</div>
		{:else if stage === 'end'}
			<p>{m.welcomeDialog_content_end_1()}</p>

			<p class="mt-1">
				{m.welcomeDialog_content_end_2()}
				<a
				href="https://discord.gg/sfuWXRfeTt"
				target="_blank"
				class="text-accent-400 hover:underline">
				{m.welcomeDialog_content_end_3()}
				</a>.
			</p>
		{/if}
	</div>
</Dialog>
