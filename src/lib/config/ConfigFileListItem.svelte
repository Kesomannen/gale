<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigFileData, ConfigSection, ConfigFile } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { Button, Collapsible } from 'bits-ui';
	import { quadOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';

	export let file: ConfigFile;
	export let selectedSection: ConfigSection | null;
	export let locked: boolean;

	export let onDeleted: () => void;
	export let onFileClicked: (file: ConfigFile) => void;
	export let onSectionClicked: (file: ConfigFileData, section: ConfigSection) => void;

	let open = false;

	$: type = file.type;
	$: isSelected = selectedSection && file.type === 'ok' && file.sections.includes(selectedSection);

	$: textColor = type === 'ok' ? 'primary-200' : type === 'err' ? 'red-400' : 'primary-400';
	$: icon = type === 'ok' ? 'mdi:chevron-down' : type === 'err' ? 'mdi:error' : 'mdi:help';

	$: shownSections =
		file.type === 'ok' ? file.sections.filter((section) => section.entries.length > 0) : [];

	async function deleteFile(evt: Event) {
		evt.stopPropagation();
		let confirmed = await confirm(`Are you sure you want to delete ${file.displayName}?`);
		if (!confirmed) return;

		await invokeCommand('delete_config_file', { file: file.relativePath });
		onDeleted();
	}
</script>

<Collapsible.Root bind:open>
	{#if type !== 'ok' || shownSections.length > 0}
		<Collapsible.Trigger
			class="group flex w-full items-center overflow-hidden py-0.5 pr-1 pl-2 text-{textColor} {isSelected
				? 'bg-primary-600 font-semibold'
				: 'hover:bg-primary-600'}"
			on:click={() => type !== 'ok' && onFileClicked(file)}
		>
			<Icon
				{icon}
				class="mr-1 shrink-0 text-lg transition-all {open && type === 'ok'
					? 'rotate-180'
					: 'rotate-0'}"
			/>

			<div class="mr-1 shrink truncate" style="direction: rtl;">
				&#x200E;
				{file.displayName ?? file.relativePath}
			</div>

			<Button.Root
				class="text-primary-400 hover:bg-primary-500 hover:text-primary-200 ml-auto hidden shrink-0 rounded-sm p-1 group-hover:flex"
				on:click={(evt) => {
					evt.stopPropagation();
					invokeCommand('open_config_file', { file: file.relativePath });
				}}
			>
				<Icon icon="mdi:open-in-new" />
			</Button.Root>

			{#if !locked}
				<Button.Root
					class="text-primary-400 hover:bg-primary-500 hover:text-primary-200 hidden shrink-0 rounded-sm p-1 group-hover:flex"
					on:click={deleteFile}
				>
					<Icon icon="mdi:delete" />
				</Button.Root>
			{/if}
		</Collapsible.Trigger>
	{/if}
	{#if file.type === 'ok' && shownSections.length > 0}
		<Collapsible.Content
			class="mb-1 flex flex-col"
			transition={slide}
			transitionConfig={{ duration: 100, easing: quadOut }}
		>
			{#each shownSections as section}
				<Button.Root
					class="truncate py-0.5 pr-2 pl-9 text-left text-sm {selectedSection === section
						? 'bg-primary-600 text-primary-200 font-semibold'
						: 'text-primary-300 hover:bg-primary-600'}"
					on:click={() => onSectionClicked(file, section)}
				>
					{section.name.length > 0 ? section.name : '<Nameless section>'}
				</Button.Root>
			{/each}
		</Collapsible.Content>
	{/if}
</Collapsible.Root>
