<script lang="ts">
	import { shortenFileSize } from '$lib/util';
	import Checklist from '../ui/Checklist.svelte';
	import Dialog from '../ui/Dialog.svelte';
	import type { ExportFile } from '$lib/types';
	import * as api from '$lib/api';
	import type { Snippet } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import HelpCard from '../ui/HelpCard.svelte';
	import profiles from '$lib/state/profile.svelte';

	type Props = {
		open: boolean;
		title: string;
		description?: Snippet;
		buttons?: Snippet;
	};

	let { open = $bindable(), title, description, buttons }: Props = $props();

	let files: ExportFile[] | null = $state(null);

	const sortedFiles = $derived.by(() => files?.toSorted((a, b) => b.size - a.size) ?? []);

	const totalSize = $derived.by(() =>
		sortedFiles.filter((file) => file.included).reduce((sum, file) => sum + file.size, 0)
	);

	async function save() {
		if (!files) return;

		const excludedFiles = files.filter((file) => !file.included).map((file) => file.path);

		await api.profile.export.setExcludedFiles(excludedFiles);
	}

	$effect(() => {
		if (open) {
			files = null;
			api.profile.export.listFiles().then((f) => {
				files = f;
			});
		}
	});

	$effect(() => {
		profiles.activeId;
		files = null;
	});
</script>

<Dialog {title} open={open && files !== null} onclose={() => (open = false)} large noscroll>
	<p class="text-gray-700 dark:text-gray-300">
		{@render description?.()}
	</p>

	{#if files}
		{#if files.length === 0}
			<HelpCard class="mt-2" title={m.exportFilesDialog_noFiles()} icon="mdi:search" />
		{:else}
			<Checklist
				title={m.exportFilesDialog_list_title()}
				items={sortedFiles}
				get={(file) => file.included}
				set={(file, _, checked) => {
					file.included = checked;
					save();
				}}
				setAll={(checked) => {
					sortedFiles.forEach((file) => (file.included = checked));
					save();
				}}
				class="mt-2"
			>
				{#snippet item({ item: file })}
					<div class="w-1/2 truncate" style="direction: rtl;" title={file.path}>
						&#x200E; {file.path}
					</div>
					<div class="mx-auto font-medium">{shortenFileSize(file.size)}</div>
				{/snippet}
			</Checklist>

			<div>
				{#if sortedFiles.length > 0}
					<div class="mt-2 text-center text-sm text-gray-600 dark:text-gray-400">
						{m.exportFilesDialog_totalSize({ size: shortenFileSize(totalSize) })}
					</div>
				{/if}
			</div>
		{/if}
	{/if}

	<div class="mt-2 flex items-center justify-end gap-1">
		{@render buttons?.()}
	</div>
</Dialog>
