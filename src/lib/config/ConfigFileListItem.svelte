<script lang="ts">
	import { invoke } from '$lib/invoke';
	import type { ConfigFileData, ConfigSection, ConfigFile } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { Collapsible } from 'bits-ui';
	import clsx from 'clsx';
	import { quadOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';

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
		let confirmed = await confirm(`Are you sure you want to delete ${file.displayName}?`);
		if (!confirmed) return;

		await invoke('delete_config_file', { file: file.relativePath });
		onDeleteClicked();
	}

	async function openFile() {
		await invoke('open_config_file', { file: file.relativePath });
	}
</script>

<Collapsible.Root bind:open>
	{#if file.type !== 'ok' || shownSections.length > 0}
		<Collapsible.Trigger
			onclick={() => file.type !== 'ok' && onFileClicked(file)}
			class={[
				colorClasses,
				isSelected ? 'bg-primary-600 font-semibold' : 'hover:bg-primary-600',
				'group flex w-full items-center overflow-hidden py-0.5 pr-1 pl-2'
			]}
		>
			<Icon
				{icon}
				class={clsx([
					open && file.type === 'ok' ? 'rotate-180' : 'rotate-0',
					'mr-1 shrink-0 text-lg transition-all'
				])}
			/>

			<div class="mr-auto shrink truncate" style="direction: rtl;">
				&#x200E;
				{file.displayName ?? file.relativePath}
			</div>

			{@render button('mdi:open-in-new', openFile)}

			{#if !locked}
				{@render button('mdi:delete', deleteFile)}
			{/if}
		</Collapsible.Trigger>
	{/if}
	{#if file.type === 'ok' && shownSections.length > 0}
		<Collapsible.Content forceMount>
			{#snippet child({ props, open })}
				{#if open}
					<div
						{...props}
						class="mb-1 flex flex-col"
						transition:slide={{ duration: 100, easing: quadOut }}
					>
						{#each shownSections as section}
							<button
								onclick={() => onSectionClicked(file, section)}
								class={[
									selectedSection === section
										? 'bg-primary-600 text-primary-200 font-semibold'
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

{#snippet button(icon: string, onclick: () => void)}
	<button
		class="text-primary-400 hover:bg-primary-500 hover:text-primary-200 hidden shrink-0 rounded-sm p-1 group-hover:flex"
		onclick={(evt) => {
			evt.stopPropagation();
			onclick();
		}}
	>
		<Icon {icon} />
	</button>
{/snippet}
