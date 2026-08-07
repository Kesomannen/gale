<script lang="ts">
	import type { Folder, Mod } from '$lib/types';
	import { modIconSrc } from '$lib/util';
	import { createSortable } from '@dnd-kit/svelte/sortable';
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';
	import ModSwitch from './ModSwitch.svelte';

	type Props = {
		folder: Folder;
		index: number;
		reorderable?: boolean;
		locked?: boolean;
		ghost?: boolean;
		dropState: 'before' | 'after' | 'folder' | null;
		mod: Snippet<[{ mod: Mod; index: number }]>;
	};

	let {
		folder = $bindable(),
		index,
		reorderable,
		locked,
		ghost = false,
		dropState = null,
		mod: modSnippet
	}: Props = $props();

	const enabled = $derived(folder.mods.some((mod) => mod.enabled ?? true));

	const sortable = createSortable({
		get id() {
			return folder.id;
		},
		get index() {
			return index;
		},
		get disabled() {
			return !reorderable;
		},
		transition: {
			duration: 0
		}
	});
</script>

<div
	{@attach sortable.attach}
	id={folder.id}
	class={[
		'relative select-none',
		ghost && 'opacity-55',
		dropState === 'folder' && 'bg-accent-500/10 ring-accent-400 ring-2'
	]}
>
	{#if dropState === 'before'}
		<div
			class="bg-accent-500/90 absolute inset-x-4 top-0 z-20 h-0.5 -translate-y-1/2 rounded-full"
		></div>
	{/if}

	<div
		class={[
			'text-primary-300 mr-2 rounded-lg border',
			folder.isExpanded ? 'border-primary-700' : 'border-transparent'
		]}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			role="button"
			tabindex="-1"
			onclick={() => (folder.isExpanded = !folder.isExpanded)}
			class={[
				!enabled && 'opacity-70',
				// can't use tailwind's odd: because the items are wrapped the virtual list item elements
				index % 2 === 1 && 'bg-primary-900/30',
				'hover:hover:bg-primary-700 grid w-full grid-cols-[1fr_auto] items-center rounded-lg p-2'
			]}
		>
			<!-- {#if leading}
				{@render leading()}
			{:else}
				<div></div>
			{/if} -->

			<div
				class={[
					'flex items-center overflow-hidden font-medium',
					enabled ? 'text-white' : 'line-through'
				]}
			>
				{#if folder.mods.length > 1}
					<div class="mr-3 grid size-12 grid-cols-2 gap-0.5">
						{#each folder.mods.slice(0, 4) as mod}
							<img src={modIconSrc(mod)} alt="" class="rounded-sm" />
						{/each}
					</div>
				{:else if folder.mods.length === 1}
					<img
						src={modIconSrc(folder.mods[0])}
						alt={folder.mods[0].name}
						class="mr-3 size-12 rounded-md"
					/>
				{/if}

				<Icon
					icon={folder.isExpanded ? 'mdi:folder-open' : 'mdi:folder'}
					class="text-primary-400 mr-2 shrink-0 text-xl"
				/>

				<span class="mr-2 truncate">{folder.name}</span>
			</div>

			<ModSwitch {enabled} {locked} ontoggle={() => {}} />
		</div>

		{#if folder.isExpanded}
			<div class="border-primary-600 pb-2 pl-8">
				{#each folder.mods as mod, modIndex (mod.uuid)}
					{@render modSnippet({ mod, index: modIndex })}
				{/each}
			</div>
		{/if}
	</div>

	{#if dropState === 'after'}
		<div
			class="bg-accent-500/90 absolute inset-x-4 bottom-0 z-20 h-0.5 translate-y-1/2 rounded-full"
		></div>
	{/if}
</div>
