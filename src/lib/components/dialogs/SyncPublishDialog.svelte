<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import InfoBox from '$lib/components/ui/InfoBox.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import * as api from '$lib/api';
	import type { SyncConfigFileInfo, SyncPublishMode } from '$lib/types';
	import { shortenFileSize } from '$lib/util';
	import { pushInfoToast } from '$lib/toast';
	import { SvelteSet } from 'svelte/reactivity';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		open: boolean;
		onPublished: () => void;
	};

	type Mode = 'mods' | 'config' | 'both';

	let { open = $bindable(), onPublished }: Props = $props();

	let mode: Mode = $state('both');
	let files: SyncConfigFileInfo[] = $state([]);
	let selected: SvelteSet<string> = $state(new SvelteSet());
	let loading = $state(false);
	let filesLoading = $state(false);

	let modeItems = $derived([
		{ value: 'mods', label: m.syncPublishDialog_mode_mods() },
		{ value: 'config', label: m.syncPublishDialog_mode_config() },
		{ value: 'both', label: m.syncPublishDialog_mode_both() }
	]);

	let modeDescription = $derived(
		{
			mods: m.syncPublishDialog_modeDescription_mods(),
			config: m.syncPublishDialog_modeDescription_config(),
			both: m.syncPublishDialog_modeDescription_both()
		}[mode]
	);

	let statusLabel = $derived({
		new: m.syncPublishDialog_status_new(),
		modified: m.syncPublishDialog_status_modified(),
		published: m.syncPublishDialog_status_published()
	});

	$effect(() => {
		if (open) {
			loadFiles();
		}
	});

	async function loadFiles() {
		filesLoading = true;
		try {
			files = await api.profile.sync.getConfigFiles();
			selected = new SvelteSet(
				files.filter((file) => file.status !== 'published').map((file) => file.path)
			);
		} catch {
			files = [];
			selected = new SvelteSet();
		} finally {
			filesLoading = false;
		}
	}

	function selectChanged() {
		selected = new SvelteSet(
			files.filter((file) => file.status !== 'published').map((file) => file.path)
		);
	}

	async function publish() {
		loading = true;
		try {
			let publishMode: SyncPublishMode =
				mode === 'mods' ? { kind: 'mods' } : { kind: mode, files: [...selected] };

			await api.profile.sync.publish(publishMode);
			pushInfoToast({ message: m.syncPublishDialog_successMessage() });
			open = false;
			onPublished();
		} finally {
			loading = false;
		}
	}
</script>

<Dialog bind:open title={m.syncPublishDialog_title()}>
	<InfoBox type="warning">{m.syncPublishDialog_warning()}</InfoBox>

	<div class="mt-3">
		<Select
			triggerClass="w-full"
			type="single"
			items={modeItems}
			value={mode}
			onValueChange={(value) => (mode = value as Mode)}
		/>
		<p class="text-primary-500 dark:text-primary-400 mt-1.5 text-sm">
			{modeDescription}
		</p>
	</div>

	{#if mode !== 'mods'}
		<div class="mt-4 flex items-center gap-2">
			<h3 class="text-primary-800 dark:text-primary-100 mr-auto font-medium">
				{m.syncPublishDialog_configHeading()}
			</h3>
			<Button color="primary" onclick={selectChanged}>
				{m.syncPublishDialog_selectChanged()}
			</Button>
			<Button color="primary" onclick={() => (selected = new SvelteSet())}>
				{m.syncPublishDialog_clear()}
			</Button>
		</div>

		{#if filesLoading}
			<Spinner class="text-primary-500 mx-auto my-8 text-3xl" />
		{:else if files.length === 0}
			<div class="text-primary-600 dark:text-primary-300 my-4 text-center">
				{m.syncPublishDialog_empty()}
			</div>
		{:else}
			<div class="mt-2 flex max-h-60 flex-col gap-1 overflow-y-auto pr-1">
				{#each files as file (file.path)}
					<div
						class="dark:bg-primary-900 bg-primary-100 flex items-center gap-2.5 rounded-md px-3 py-1.5"
					>
						<Checkbox
							checked={selected.has(file.path)}
							onCheckedChange={(checked) => {
								if (checked) {
									selected.add(file.path);
								} else {
									selected.delete(file.path);
								}
							}}
						/>
						<span class="text-primary-800 dark:text-primary-100 truncate" title={file.path}>
							{file.path}
						</span>
						<span class="text-primary-500 dark:text-primary-400 shrink-0 text-sm">
							{shortenFileSize(file.size)}
						</span>
						<span
							class={[
								'ml-auto shrink-0 rounded px-2 py-0.5 text-xs font-medium',
								{
									new: 'bg-accent-600 text-white',
									modified: 'text-primary-900 bg-yellow-500',
									published:
										'bg-primary-300 text-primary-600 dark:bg-primary-700 dark:text-primary-300'
								}[file.status]
							]}
						>
							{statusLabel[file.status]}
						</span>
					</div>
				{/each}
			</div>
		{/if}
	{/if}

	<div class="mt-4 flex justify-end">
		<Button
			onclick={publish}
			{loading}
			disabled={mode === 'config' && selected.size === 0}
			icon="mdi:cloud-upload"
		>
			{m.syncPublishDialog_submit()}
		</Button>
	</div>
</Dialog>
