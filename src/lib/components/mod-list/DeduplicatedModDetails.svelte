<script lang="ts">
	import {
		Backend,
		type ContextItem,
		type DeduplicatedMod,
		type Mod,
		type ModContextItem
	} from '$lib/types';
	import type { Snippet } from 'svelte';
	import ModDetails from './ModDetails.svelte';
	import TabsMenu from '../ui/TabsMenu.svelte';
	import {
		capitalize,
		extractDeduplicatedMod,
		getPreferredBackend,
		resolveModContextItems
	} from '$lib/util';

	type Props = {
		mod: DeduplicatedMod<Mod>;
		contextItems: ModContextItem[];
		locked: boolean;
		onclose: () => void;
		header?: Snippet;
		children?: Snippet<[{ mod: Mod }]>;
	};

	let { mod, header: headerProp, children, contextItems, locked, onclose }: Props = $props();

	const availableBackends = $derived.by(() => {
		const backends: Backend[] = [];
		if (mod.thunderstore) backends.push(Backend.Thunderstore);
		if (mod.hexium) backends.push(Backend.Hexium);
		return backends;
	});

	let selectedBackend = $derived(getPreferredBackend(mod));
	const selectedMod = $derived(extractDeduplicatedMod(mod, selectedBackend));
</script>

{#if selectedMod}
	<ModDetails
		mod={selectedMod}
		hideBackend={availableBackends.length > 1}
		contextItems={resolveModContextItems(contextItems, selectedMod, locked)}
		{onclose}
	>
		{#snippet header()}
			{@render headerProp?.()}

			{#if availableBackends.length > 1}
				<TabsMenu
					class="mt-2 mb-4"
					bind:value={selectedBackend}
					options={availableBackends.map((backend) => ({
						value: backend,
						label: capitalize(backend)
					}))}
				/>
			{/if}
		{/snippet}

		{@render children?.({ mod: selectedMod })}
	</ModDetails>
{/if}
