<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigFile, ConfigSection, LoadFileResult } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { Button, Collapsible } from 'bits-ui';
	import { quadOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';

	export let file: LoadFileResult;
	export let selectedSection: ConfigSection | undefined;
	export let onDeleted: () => void;
	export let onFileClicked: (file: LoadFileResult) => void;
	export let onSectionClicked: (file: ConfigFile, section: ConfigSection) => void;

	let open = false;

	$: type = file.type;
	$: isSelected = selectedSection && file.type === 'ok' && file.sections.includes(selectedSection);

	$: textColor = type === 'ok' ? 'gray-200' : type === 'err' ? 'red-400' : 'gray-400';
	$: icon = type === 'ok' ? 'mdi:chevron-down' : type === 'err' ? 'mdi:error' : 'mdi:help';

	$: shownSections =
		file.type === 'ok'
			? file.sections.filter(
					({ entries }) => entries.filter(({ type }) => type === 'normal').length > 0
				)
			: [];
</script>

<Collapsible.Root bind:open>
	{#if type !== 'ok' || shownSections.length > 0}
		<Collapsible.Trigger
			class="group flex w-full items-center overflow-hidden py-0.5 pl-2 pr-1 text-{textColor} {isSelected
				? 'bg-gray-600 font-semibold'
				: 'hover:bg-gray-600'}"
			on:click={() => type !== 'ok' && onFileClicked(file)}
		>
			<Icon
				{icon}
				class="mr-1 flex-shrink-0 text-lg transition-all {open && type === 'ok'
					? 'rotate-180'
					: 'rotate-0'}"
			/>

			<div class="mr-1 flex-shrink truncate" style="direction: rtl;">
				&#x200E;
				{file.displayName ?? file.relativePath}
			</div>

			<Button.Root
				class="ml-auto hidden flex-shrink-0 rounded p-1 text-gray-400 hover:bg-gray-500 hover:text-gray-200 group-hover:flex"
				on:click={(evt) => {
					evt.stopPropagation();
					invokeCommand('open_config_file', { file: file.relativePath });
				}}
			>
				<Icon icon="mdi:open-in-new" />
			</Button.Root>

			<Button.Root
				class="hidden flex-shrink-0 rounded p-1 text-gray-400 hover:bg-gray-500 hover:text-gray-200 group-hover:flex"
				on:click={async (evt) => {
					evt.stopPropagation();
					await invokeCommand('delete_config_file', { file: file.relativePath });
					onDeleted();
				}}
			>
				<Icon icon="mdi:delete" />
			</Button.Root>
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
					class="truncate py-0.5 pl-9 pr-2 text-left text-sm {selectedSection === section
						? 'bg-gray-600 font-semibold text-gray-200'
						: 'text-gray-300 hover:bg-gray-600'}"
					on:click={() => onSectionClicked(file, section)}
				>
					{section.name.length > 0 ? section.name : '<Nameless section>'}
				</Button.Root>
			{/each}
		</Collapsible.Content>
	{/if}
</Collapsible.Root>
