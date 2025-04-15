<script lang="ts" context="module">
	import { writable } from 'svelte/store';

	const threshold = writable(0);

	activeProfile.subscribe(() => {
		threshold.set(0);
	});
</script>

<script lang="ts">
	import Checklist from '$lib/components/Checklist.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import type { AvailableUpdate } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import ModCard from './ModCard.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { invokeCommand } from '$lib/invoke';
	import BigButton from '$lib/components/BigButton.svelte';
	import { activeProfile, refreshProfiles } from '$lib/stores';

	export let updates: AvailableUpdate[];

	let popupOpen = false;
	let include: Map<AvailableUpdate, boolean> = new Map();

	$: shownUpdates = updates.filter((update) => !update.ignore);
	$: if (popupOpen && shownUpdates.length === 0) {
		popupOpen = false;
	}

	async function updateAll() {
		let uuids = shownUpdates
			.filter((update) => include.get(update) ?? true)
			.map((update) => update.packageUuid);

		popupOpen = false;

		await invokeCommand('update_mods', { uuids, respectIgnored: true });
		await refreshProfiles();
	}
</script>

{#if shownUpdates.length > $threshold}
	<div class="bg-accent-700 text-accent-100 mr-3 mb-1 flex items-center rounded-lg py-1 pr-1 pl-3">
		<Icon icon="mdi:arrow-up-circle" class="mr-2 text-xl" />
		There {shownUpdates.length === 1 ? 'is' : 'are'}
		<b class="mx-1">{shownUpdates.length}</b>
		{shownUpdates.length === 1 ? ' update' : ' updates'} available.
		<Button.Root
			class="hover:text-accent-200 ml-1 font-semibold text-white hover:underline"
			on:click={() => (popupOpen = true)}
		>
			Update all?
		</Button.Root>

		<Button.Root
			class="hover:bg-accent-600 ml-auto rounded-md p-1 text-xl"
			on:click={() => ($threshold = shownUpdates.length)}
		>
			<Icon icon="mdi:close" />
		</Button.Root>
	</div>
{/if}

<ConfirmPopup title="Confirm update" bind:open={popupOpen}>
	Select which mods to update:

	<Checklist
		title="Update all"
		items={shownUpdates}
		class="mt-1"
		maxHeight="sm"
		let:item={update}
		get={(update, _) => include.get(update) ?? true}
		set={(update, _, value) => {
			include.set(update, value);
			include = include; // force reactivity
		}}
	>
		<ModCard fullName={update.fullName} showVersion={false} />

		<span class="text-light text-primary-400 ml-auto pl-1">{update.old}</span>
		<Icon icon="mdi:arrow-right" class="text-primary-400 mx-1.5 text-lg" />
		<span class="text-accent-400 text-lg font-semibold">{update.new}</span>

		<Tooltip text="Ignore this update in the 'Update all' list." side="left" sideOffset={-2}>
			<Button.Root
				class="text-primary-400 hover:bg-primary-700 hover:text-primary-200 ml-2 rounded-sm p-1.5"
				on:click={() => {
					update.ignore = true;
					updates = updates; // force reactivity

					include.delete(update);
					include = include; // force reactivity

					invokeCommand('ignore_update', { versionUuid: update.versionUuid });
				}}><Icon icon="mdi:notifications-off" /></Button.Root
			>
		</Tooltip>
	</Checklist>

	<svelte:fragment slot="buttons">
		<BigButton color="accent" on:click={updateAll}>Update mods</BigButton>
	</svelte:fragment>
</ConfirmPopup>
