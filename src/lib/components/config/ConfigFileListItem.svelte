<script lang="ts">
	import * as api from '$lib/api';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import type { ConfigFileData, ConfigSection, ConfigFile } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { Collapsible } from 'bits-ui';
	import clsx from 'clsx';

	type Props = {
		file: ConfigFile;
		duplicate: boolean;
		selectedSection: ConfigSection | null;
		locked: boolean;
		onDeleteClicked: () => void;
		onFileClicked: (file: ConfigFile) => void;
		onSectionClicked: (file: ConfigFileData, section: ConfigSection) => void;
	};

	let {
		file,
		selectedSection,
		duplicate,
		locked,
		onDeleteClicked,
		onFileClicked,
		onSectionClicked
	}: Props = $props();

	let open = $state(false);

	let isSelected = $derived(
		selectedSection && file.type === 'ok' && file.sections.includes(selectedSection)
	);

	let { colorClasses, icon } = $derived(
		{
			ok: { colorClasses: 'text-primary-200', icon: 'mdi:chevron-down' },
			err: { colorClasses: 'text-red-400', icon: 'mdi:error' },
			unsupported: { colorClasses: 'text-primary-400', icon: 'mdi:help' }
		}[file.type]
	);

	let shownSections = $derived(
		file.type === 'ok' ? file.sections.filter((section) => section.entries.length > 0) : []
	);

	let fileLabel = $derived(() => {
		if (!file.displayName) {
			return { name: file.relativePath, disambiguator: null };
		}

		if (duplicate) {
			const firstFolder = file.relativePath.split('/')[0];
			if (firstFolder !== file.displayName) {
				return { name: file.displayName, disambiguator: firstFolder };
			}
		}

		return { name: file.displayName, disambiguator: null };
	});

	async function deleteFile() {
		let confirmed = await confirm(`Are you sure you want to delete ${file.displayName}?`);
		if (!confirmed) return;

		await api.config.deleteFile(file);
		onDeleteClicked();
	}

	async function openFile() {
		await api.config.openFile(file);
	}
</script>

<Collapsible.Root bind:open>
	{#if file.type !== 'ok' || shownSections.length > 0}
		<Collapsible.Trigger
			onclick={() => file.type !== 'ok' && onFileClicked(file)}
			class={[
				colorClasses,
				isSelected ? 'bg-primary-600 font-semibold' : 'hover:bg-primary-600',
				'group flex w-full items-center overflow-hidden px-2 py-0.5'
			]}
		>
			<Icon
				{icon}
				class={clsx([
					open && file.type === 'ok' ? 'rotate-180' : 'rotate-0',
					'mr-1 shrink-0 text-lg'
				])}
			/>

			<div class="mr-auto shrink truncate" style="direction: rtl;">
				&#x200E;
				{fileLabel().name}
				{#if fileLabel().disambiguator}
					<span class="text-primary-400">
						({fileLabel().disambiguator})
					</span>
				{/if}
			</div>

			<IconButton
				label="Open in external program"
				icon="mdi:open-in-new"
				class="ml-2 hidden group-hover:block"
				onclick={(evt) => {
					evt.preventDefault();
					openFile();
				}}
			/>

			{#if !locked}
				<IconButton
					label="Trash file"
					icon="mdi:delete"
					class="hidden group-hover:block"
					onclick={(evt) => {
						evt.preventDefault();
						deleteFile();
					}}
				/>
			{/if}
		</Collapsible.Trigger>
	{/if}
	{#if file.type === 'ok' && shownSections.length > 0}
		<Collapsible.Content forceMount>
			{#snippet child({ props, open })}
				{#if open}
					<div {...props} class="mb-1 flex flex-col">
						{#each shownSections as section}
							<button
								onclick={() => onSectionClicked(file, section)}
								class={[
									selectedSection === section
										? 'bg-primary-600 text-primary-200 font-medium'
										: 'text-primary-300 hover:bg-primary-600',
									'truncate py-0.5 pr-2 pl-9 text-left text-sm'
								]}
							>
								{section.name.length > 0 ? section.name : '<Nameless section>'}
							</button>
						{/each}
					</div>
				{/if}
			{/snippet}
		</Collapsible.Content>
	{/if}
</Collapsible.Root>
