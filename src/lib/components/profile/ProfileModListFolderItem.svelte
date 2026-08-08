<script lang="ts">
	import type { Folder, Mod, ModContextItem } from '$lib/types';
	import { modIconSrc } from '$lib/util';
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';
	import ModSwitch from './ModSwitch.svelte';
	import ModItemContext from '../mod-list/ModItemContext.svelte';
	import ProfileModListItem from './ProfileModListItem.svelte';

	type Props = {
		folder: Folder;
		index: number;
		reorderable?: boolean;
		locked?: boolean;
		contextItems: ModContextItem<Folder>[];
		mod: Snippet<[{ mod: Mod; index: number }]>;
		ontoggle?: (newState: boolean) => void;
		dropState?: 'before' | 'after' | 'folder' | null;
		ghost?: boolean;
	};

	let {
		folder = $bindable(),
		index,
		reorderable,
		locked = false,
		contextItems,
		mod: modSnippet,
		ontoggle,
		dropState = null,
		ghost
	}: Props = $props();

	const enabled = $derived(
		folder.mods.length === 0 || folder.mods.some((mod) => mod.enabled ?? true)
	);
</script>

<ProfileModListItem id={folder.id} {index} {reorderable} {dropState} {ghost}>
	<ModItemContext id={folder.id} subject={folder} {locked} {contextItems}>
		<div
			class={[
				'text-primary-300 rounded-lg border',
				folder.isExpanded ? 'border-primary-600' : 'border-transparent'
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
				<div
					class={[
						'flex items-center overflow-hidden font-medium',
						enabled ? 'text-white' : 'line-through'
					]}
				>
					<div class="mr-3 grid size-12 grid-cols-2 gap-0.5">
						{#each folder.mods.slice(0, 4) as mod}
							<img src={modIconSrc(mod)} alt="" class="rounded-sm" />
						{/each}
					</div>

					<Icon
						icon={folder.isExpanded ? 'mdi:folder-open' : 'mdi:folder'}
						class="text-primary-400 mr-2 shrink-0 text-xl"
					/>

					<span class="mr-2 truncate">{folder.name}</span>
				</div>

				<ModSwitch {enabled} {locked} ontoggle={(newState) => ontoggle?.(newState)} />
			</div>

			{#if folder.isExpanded}
				<div class="border-primary-600 pb-2 pl-8">
					{#each folder.mods as mod, modIndex (mod.uuid)}
						{@render modSnippet({ mod, index: modIndex })}
					{:else}{/each}
				</div>
			{/if}
		</div>
	</ModItemContext>
</ProfileModListItem>
