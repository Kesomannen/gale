<script lang="ts">
	import Checklist from '$lib/components/ui/Checklist.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import type { AvailableUpdate } from '$lib/types';
	import Icon from '@iconify/svelte';
	import ModCard from '../ui/ModCard.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import * as api from '$lib/api';
	import Button from '$lib/components/ui/Button.svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import profiles from '$lib/state/profile.svelte';
	import { updateBanner } from '$lib/state/misc.svelte';

	type Props = {
		updates: AvailableUpdate[];
	};

	let { updates }: Props = $props();

	let dialogOpen = $state(false);
	let include: SvelteMap<AvailableUpdate, boolean> = $state(new SvelteMap());

	let shownUpdates = $derived(updates.filter((update) => !update.ignore));

	$effect(() => {
		if (dialogOpen && shownUpdates.length === 0) {
			dialogOpen = false;
		}
	});

	async function updateAll() {
		let uuids = shownUpdates
			.filter((update) => include.get(update) ?? true)
			.map((update) => update.packageUuid);

		dialogOpen = false;

		await api.profile.update.mods(uuids, true);
		await profiles.refresh();
	}
</script>

{#if shownUpdates.length > updateBanner.threshold}
	<div class="bg-accent-700 text-accent-100 mr-3 mb-1 flex items-center rounded-lg py-1 pr-1 pl-3">
		<Icon icon="mdi:arrow-up-circle" class="mr-2 text-xl" />
		There {shownUpdates.length === 1 ? 'is' : 'are'}
		<b class="mx-1">{shownUpdates.length}</b>
		{shownUpdates.length === 1 ? ' update' : ' updates'} available.
		<button
			class="hover:text-accent-200 ml-1 font-semibold text-black hover:underline"
			onclick={() => (dialogOpen = true)}
		>
			Update all?
		</button>

		<button
			class="hover:bg-accent-600 ml-auto rounded-md p-1 text-xl"
			onclick={() => (updateBanner.threshold = shownUpdates.length)}
		>
			<Icon icon="mdi:close" />
		</button>
	</div>
{/if}

<ConfirmDialog title="Confirm update" bind:open={dialogOpen}>
	Select which mods to update:

	<Checklist
		title="Update all"
		items={shownUpdates}
		class="mt-1"
		maxHeight="sm"
		get={(update, _) => include.get(update) ?? true}
		set={(update, _, value) => include.set(update, value)}
	>
		{#snippet item({ item: update })}
			<ModCard fullName={update.fullName} showVersion={false} />

			<span class="text-light text-primary-400 ml-auto pl-1">{update.old}</span>
			<Icon icon="mdi:arrow-right" class="text-primary-400 mx-1.5 text-lg" />
			<span class="text-accent-400 text-lg font-semibold">{update.new}</span>

			<Tooltip text="Ignore this update in the 'Update all' list." side="left" sideOffset={-2}>
				<button
					class="text-primary-400 hover:bg-primary-700 hover:text-primary-200 ml-2 rounded-sm p-1.5"
					onclick={() => {
						update.ignore = true;
						include.delete(update);

						api.profile.update.ignore(update.versionUuid);
					}}><Icon icon="mdi:notifications-off" /></button
				>
			</Tooltip>
		{/snippet}
	</Checklist>

	{#snippet buttons()}
		<Button color="accent" icon="mdi:download" onclick={updateAll}>Update mods</Button>
	{/snippet}
</ConfirmDialog>
