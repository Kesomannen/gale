<script lang="ts">
	import { invoke } from '$lib/invoke';
	import type { ConfigFileData, ConfigSection, ConfigFile } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { Collapsible } from 'bits-ui';

	type Props = {
		file: ConfigFile;
		selectedSection: ConfigSection | null;
		locked: boolean;
		onDeleted: () => void;
		onFileClicked: (file: ConfigFile) => void;
		onSectionClicked: (file: ConfigFileData, section: ConfigSection) => void;
	};

	let { file, selectedSection, locked, onDeleted, onFileClicked, onSectionClicked }: Props =
		$props();

	let open = $state(false);

	let type = $derived(file.type);
	let isSelected = $derived(
		selectedSection && file.type === 'ok' && file.sections.includes(selectedSection)
	);

	let textColor = $derived(
		type === 'ok' ? 'primary-200' : type === 'err' ? 'red-400' : 'primary-400'
	);
	let icon = $derived(
		type === 'ok' ? 'mdi:chevron-down' : type === 'err' ? 'mdi:error' : 'mdi:help'
	);

	let shownSections = $derived(
		file.type === 'ok' ? file.sections.filter((section) => section.entries.length > 0) : []
	);

	async function deleteFile(evt: Event) {
		evt.stopPropagation();
		let confirmed = await confirm(`Are you sure you want to delete ${file.displayName}?`);
		if (!confirmed) return;

		await invoke('delete_config_file', { file: file.relativePath });
		onDeleted();
	}
</script>

<Collapsible.Root bind:open>
	{#if type !== 'ok' || shownSections.length > 0}
		<Collapsible.Trigger
			class={[
				isSelected ? 'bg-primary-600 font-semibold' : 'hover:bg-primary-600',
				`text-${textColor}`,
				'group flex w-full items-center overflow-hidden py-0.5 pr-1 pl-2'
			]}
			onclick={() => type !== 'ok' && onFileClicked(file)}
		>
			<Icon
				{icon}
				class={[
					open && type === 'ok' ? 'rotate-180' : 'rotate-0',
					'mr-1 shrink-0 text-lg transition-all'
				]}
			/>
			<div class="mr-1 shrink truncate" style="direction: rtl;">
				&#x200E;
				{file.displayName ?? file.relativePath}
			</div>

			<button
				class="text-primary-400 hover:bg-primary-500 hover:text-primary-200 ml-auto hidden shrink-0 rounded-sm p-1 group-hover:flex"
				onclick={(evt) => {
					evt.stopPropagation();
					invoke('open_config_file', { file: file.relativePath });
				}}
			>
				<Icon icon="mdi:open-in-new" />
			</button>

			{#if !locked}
				<button
					class="text-primary-400 hover:bg-primary-500 hover:text-primary-200 hidden shrink-0 rounded-sm p-1 group-hover:flex"
					onclick={deleteFile}
				>
					<Icon icon="mdi:delete" />
				</button>
			{/if}
		</Collapsible.Trigger>
	{/if}
	{#if file.type === 'ok' && shownSections.length > 0}
		<Collapsible.Content class="mb-1 flex flex-col">
			{#each shownSections as section}
				<button
					class={[
						selectedSection === section
							? 'bg-primary-600 text-primary-200 font-semibold'
							: 'text-primary-300 hover:bg-primary-600',
						'truncate py-0.5 pr-2 pl-9 text-left text-sm'
					]}
					onclick={() => onSectionClicked(file, section)}
				>
					{section.name.length > 0 ? section.name : '<Nameless section>'}
				</button>
			{/each}
		</Collapsible.Content>
	{/if}
</Collapsible.Root>
