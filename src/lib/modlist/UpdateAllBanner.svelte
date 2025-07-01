<script lang="ts" module>
	import { writable } from 'svelte/store';

	const threshold = writable(0);

	activeProfile.subscribe(() => {
		threshold.set(0);
	});
</script>

<script lang="ts">
	import Checklist from '$lib/components/Checklist.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import type { AvailableUpdate } from '$lib/types';
	import Icon from '@iconify/svelte';
	import ModCard from './ModCard.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { invoke } from '$lib/invoke';
	import BigButton from '$lib/components/Button.svelte';
	import { activeProfile, refreshProfiles } from '$lib/stores.svelte';

	type Props = {
		updates: AvailableUpdate[];
	};

	let { updates }: Props = $props();

	let popupOpen = $state(false);
	let include: Map<AvailableUpdate, boolean> = $state(new Map());

	let shownUpdates = $derived(updates.filter((update) => !update.ignore));

	$effect(() => {
		if (popupOpen && shownUpdates.length === 0) {
			popupOpen = false;
		}
	});

	async function updateAll() {
		let uuids = shownUpdates
			.filter((update) => include.get(update) ?? true)
			.map((update) => update.packageUuid);

		popupOpen = false;

		await invoke('update_mods', { uuids, respectIgnored: true });
		await refreshProfiles();
	}
</script>

{#if shownUpdates.length > $threshold}
	<div class="bg-accent-700 text-accent-100 mr-3 mb-1 flex items-center rounded-lg py-1 pr-1 pl-3">
		<Icon icon="mdi:arrow-up-circle" class="mr-2 text-xl" />
		There {shownUpdates.length === 1 ? 'is' : 'are'}
		<b class="mx-1">{shownUpdates.length}</b>
		{shownUpdates.length === 1 ? ' update' : ' updates'} available.
		<button
			class="hover:text-accent-200 ml-1 font-semibold text-white hover:underline"
			onclick={() => (popupOpen = true)}
		>
			Update all?
		</button>

		<button
			class="hover:bg-accent-600 ml-auto rounded-md p-1 text-xl"
			onclick={() => ($threshold = shownUpdates.length)}
		>
			<Icon icon="mdi:close" />
		</button>
	</div>
{/if}

<ConfirmPopup title="Confirm update" bind:open={popupOpen}>
	Select which mods to update:

	<Checklist
		title="Update all"
		items={shownUpdates}
		class="mt-1"
		maxHeight="sm"
		get={(update, _) => include.get(update) ?? true}
		set={(update, _, value) => {
			include.set(update, value);
			include = include; // force reactivity
		}}
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
						updates = updates; // force reactivity

						include.delete(update);
						include = include; // force reactivity

						invoke('ignore_update', { versionUuid: update.versionUuid });
					}}><Icon icon="mdi:notifications-off" /></button
				>
			</Tooltip>
		{/snippet}
	</Checklist>

	{#snippet buttons()}
		<BigButton color="accent" onclick={updateAll}>Update mods</BigButton>
	{/snippet}
</ConfirmPopup>
