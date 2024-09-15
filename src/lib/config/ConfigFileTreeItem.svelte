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

	$: textColor = type === 'ok' ? 'slate-200' : type === 'err' ? 'red-400' : 'slate-400';
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
			class="flex items-center group w-full overflow-hidden px-2 py-0.5
                text-{textColor}
                {isSelected ? 'bg-slate-600 font-semibold' : 'hover:bg-slate-600'}"
			on:click={() => type !== 'ok' && onFileClicked(file)}
		>
			<Icon
				{icon}
				class="text-lg transition-all mr-1 flex-shrink-0 
                    {open && type === 'ok' ? 'rotate-180' : 'rotate-0'}"
			/>

			<div class="truncate flex-shrink mr-1" style="direction: rtl;">
				&#x200E;
				{file.displayName ?? file.relativePath}
			</div>

			<Button.Root
				class="flex-shrink-0 hidden group-hover:inline text-slate-400 p-1 rounded hover:text-slate-200 hover:bg-slate-500 ml-auto"
				on:click={(evt) => {
					evt.stopPropagation();
					invokeCommand('open_config_file', { file: file.relativePath });
				}}
			>
				<Icon icon="mdi:open-in-new" />
			</Button.Root>
			<Button.Root
				class="flex-shrink-0 hidden group-hover:inline text-slate-400 p-1 rounded hover:text-slate-200 hover:bg-slate-500"
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
			class="flex flex-col mb-1"
			transition={slide}
			transitionConfig={{ duration: 100, easing: quadOut }}
		>
			{#each shownSections as section}
				<Button.Root
					class="pl-9 pr-2 py-0.5 text-left truncate text-sm
				{selectedSection === section
						? 'text-slate-200 bg-slate-600 font-semibold'
						: 'text-slate-300 hover:bg-slate-600'}"
					on:click={() => onSectionClicked(file, section)}
				>
					{section.name}
				</Button.Root>
			{/each}
		</Collapsible.Content>
	{/if}
</Collapsible.Root>
