<script lang="ts">
	import { Backend, type DeduplicatedMod, type Mod, type ModContextItem } from '$lib/types';
	import type { Snippet } from 'svelte';
	import ModDetails from './ModDetails.svelte';
	import TabsMenu from '../ui/TabsMenu.svelte';
	import { capitalize } from '$lib/util';

	type Props = {
		mod: DeduplicatedMod<Mod>;
		contextItems?: ModContextItem[];
		locked: boolean;
		onclose: () => void;
		header?: Snippet;
		children?: Snippet<[{ mod: Mod }]>;
	};

	let { mod, header, children, ...props }: Props = $props();

	const availableBackends = $derived.by(() => {
		const backends: Backend[] = [];
		if (mod.thunderstore) backends.push(Backend.Thunderstore);
		if (mod.hexium) backends.push(Backend.Hexium);
		return backends;
	});

	$inspect(availableBackends);

	let selectedBackend: Backend = $derived(availableBackends[0]);

	const selectedMod = $derived.by(() => {
		if (selectedBackend === Backend.Thunderstore) {
			return mod.thunderstore;
		} else if (selectedBackend === Backend.Hexium) {
			return mod.hexium;
		} else {
			return null;
		}
	});
</script>

{#if selectedMod}
	<ModDetails mod={selectedMod} {...props}>
		{#snippet header()}
			{@render header?.()}

			{#if availableBackends.length > 1}
				<TabsMenu
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
