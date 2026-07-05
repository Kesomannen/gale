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
	import { updateBanner } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import { DropdownMenu } from 'bits-ui';
	import ContextMenuContent from '../ui/ContextMenuContent.svelte';

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
	}

	function ignoreUpdate(update: AvailableUpdate) {
		update.ignore = true;
		include.delete(update);
	}
</script>

{#if shownUpdates.length > updateBanner.threshold}
	<div class="bg-accent-700 text-accent-100 mb-1 flex items-center rounded-lg py-1 pr-1 pl-3">
		<Icon icon="mdi:arrow-up-circle" class="mr-2 text-xl" />
		{m.updateAllBanner_content({ count: shownUpdates.length })}

		<button
			class="hover:text-accent-200 ml-1 font-semibold text-white hover:underline"
			onclick={() => (dialogOpen = true)}
		>
			{m.updateAllBanner_button()}
		</button>

		<button
			class="hover:bg-accent-600 ml-auto rounded-md p-1 text-xl"
			onclick={() => (updateBanner.threshold = shownUpdates.length)}
		>
			<Icon icon="mdi:close" />
		</button>
	</div>
{/if}

<ConfirmDialog title={m.updateAllBanner_dialog_title()} bind:open={dialogOpen}>
	{m.updateAllBanner_dialog_content()}

	<Checklist
		title={m.updateAllBanner_dialog_list_title()}
		items={shownUpdates}
		class="mt-1"
		maxHeight="sm"
		get={(update, _) => include.get(update) ?? true}
		set={(update, _, value) => include.set(update, value)}
	>
		{#snippet item({ item: update })}
			<ModCard fullName={update.fullName} showVersion={false} backend={update.backend} />

			<span class="text-light text-primary-400 ml-auto pl-1">{update.old}</span>
			<Icon icon="mdi:arrow-right" class="text-primary-400 mx-1.5 text-lg" />
			<span class="text-accent-400 text-lg font-semibold">{update.new}</span>

			<DropdownMenu.Root>
				<DropdownMenu.Trigger
					class="text-primary-400 hover:bg-primary-700 hover:text-primary-200 ml-2 rounded-sm p-1.5"
				>
					<Icon icon="mdi:notifications-off" />
				</DropdownMenu.Trigger>
				<ContextMenuContent
					type="dropdown"
					items={[
						{
							label: m.updateAllBanner_dialog_list_ignore_version(),
							onclick: () => {
								ignoreUpdate(update);
								api.profile.update.ignore(update.versionUuid);
							}
						},
						{
							label: m.updateAllBanner_dialog_list_ignore_package(),
							onclick: () => {
								ignoreUpdate(update);
								api.profile.update.ignorePackage(update.packageUuid);
							}
						}
					]}
				/>
			</DropdownMenu.Root>
		{/snippet}
	</Checklist>

	{#snippet buttons()}
		<Button color="accent" icon="mdi:download" onclick={updateAll}
			>{m.updateAllBanner_dialog_button()}</Button
		>
	{/snippet}
</ConfirmDialog>
