<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigFile, ConfigSection, LoadFileResult } from '$lib/models';
	import { configDisplayName } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Button, Collapsible } from 'bits-ui';
	import { quadOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';

	export let file: LoadFileResult;
	export let selectedSection: ConfigSection | undefined;
	export let onDeleted: () => void;
	export let onErrorFileClicked: (file: LoadFileResult) => void;
	export let onSectionClicked: (file: ConfigFile, section: ConfigSection) => void;

	let open = false;

	$: isError = file.type !== 'ok';
	$: isSelected =
		selectedSection && file.type === 'ok' && file.content.sections.includes(selectedSection);
</script>

<Collapsible.Root bind:open>
	<Collapsible.Trigger
		class="flex items-center group w-full overflow-hidden px-2 py-0.5
                {isError ? 'text-red-400' : 'text-slate-200'} 
                {isSelected ? 'bg-slate-600 font-semibold' : 'hover:bg-slate-600'}"
		on:click={() => {
			if (isError) {
				onErrorFileClicked(file);
			}
		}}
	>
		<Icon
			icon={isError ? 'mdi:error' : 'mdi:chevron-down'}
			class="text-lg transition-all mr-1 flex-shrink-0 
                    {open ? 'rotate-180' : 'rotate-0'}"
		/>

		<div class="truncate flex-shrink mr-1" style="direction: rtl;">
			{configDisplayName(file)}
		</div>

		<Button.Root
			class="flex-shrink-0 hidden group-hover:inline text-slate-400 p-1 rounded hover:text-slate-200 hover:bg-slate-500 ml-auto"
			on:click={(evt) => {
				evt.stopPropagation();
				invokeCommand('open_config_file', { file: file.content.name });
			}}
		>
			<Icon icon="mdi:open-in-new" />
		</Button.Root>
		<Button.Root
			class="flex-shrink-0 hidden group-hover:inline text-slate-400 p-1 rounded hover:text-slate-200 hover:bg-slate-500"
			on:click={async (evt) => {
				evt.stopPropagation();
				await invokeCommand('delete_config_file', { file: file.content.name });
				onDeleted();
			}}
		>
			<Icon icon="mdi:delete" />
		</Button.Root>
	</Collapsible.Trigger>
	{#if file.type == 'ok'}
		<Collapsible.Content
			class="flex flex-col mb-1"
			transition={slide}
			transitionConfig={{ duration: 100, easing: quadOut }}
		>
			{#each file.content.sections as section}
				<Button.Root
					class="pl-9 pr-2 py-0.5 text-left truncate text-sm
                    {selectedSection === section
						? 'text-slate-200 bg-slate-600 font-semibold'
						: 'text-slate-300 hover:bg-slate-600'}"
					on:click={() => onSectionClicked(file.content, section)}
				>
					{section.name}
				</Button.Root>
			{/each}
		</Collapsible.Content>
	{/if}
</Collapsible.Root>
