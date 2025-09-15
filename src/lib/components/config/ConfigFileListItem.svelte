<script lang="ts">
	import * as api from '$lib/api';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { ConfigFileData, ConfigSection, ConfigFile } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { Collapsible } from 'bits-ui';
	import clsx from 'clsx';

	type Props = {
		file: ConfigFile;
		selectedSection: ConfigSection | null;
		locked: boolean;
		onDeleteClicked: () => void;
		onFileClicked: (file: ConfigFile) => void;
		onSectionClicked: (file: ConfigFileData, section: ConfigSection) => void;
	};

	let { file, selectedSection, locked, onDeleteClicked, onFileClicked, onSectionClicked }: Props =
		$props();

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

	async function deleteFile() {
		let confirmed = await confirm(m.configFileListItem_deleteFile_confirm({name: file.displayName ?? file.relativePath}));
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
				{file.displayName ?? file.relativePath}
			</div>

			<IconButton
				label={m.configFileListItem_button_openFile()}
				icon="mdi:open-in-new"
				class="ml-2 hidden group-hover:block"
				onclick={(evt) => {
					evt.preventDefault();
					openFile();
				}}
			/>

			{#if !locked}
				<IconButton
					label={m.configFileListItem_button_deleteFile()}
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
								{section.name.length > 0 ? section.name : m.configFileListItem_nameLess()}
							</button>
						{/each}
					</div>
				{/if}
			{/snippet}
		</Collapsible.Content>
	{/if}
</Collapsible.Root>
