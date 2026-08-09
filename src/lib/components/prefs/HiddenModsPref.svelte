<script lang="ts">
	import Info from '$lib/components/ui/Info.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import Dialog from '../ui/Dialog.svelte';
	import Icon from '@iconify/svelte';
	import type { Dependant } from '$lib/types';
	import { onMount } from 'svelte';
	import ModCardList from '../ui/ModCardList.svelte';
	import IconButton from '../ui/IconButton.svelte';
	import ResetButton from '../ui/ResetButton.svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { pushInfoToast } from '$lib/toast';

	type Props = {};

	let {}: Props = $props();

	let hiddenMods: Dependant[] = $state([]);
	let hasHiddenMods = $derived(hiddenMods.length > 0);

	let dialogOpen = $state(false);

	onMount(async () => {
		hiddenMods = await api.profile.getHiddenMods();
	});

	async function unHideMod(mod: Dependant) {
		await api.profile.toggleHiddenMod(mod.uuid);
		hiddenMods = hiddenMods.filter((other) => other.uuid !== mod.uuid);

		if (hiddenMods.length === 0) {
			dialogOpen = false;
		}
	}

	async function onResetClicked() {
		const confirmed = await confirm(m.hiddenModsPref_reset_message({ count: hiddenMods.length }));

		if (!confirmed) {
			return;
		}

		const count = hiddenMods.length;

		for (const mod of hiddenMods) {
			await api.profile.toggleHiddenMod(mod.uuid);
		}

		hiddenMods = [];

		pushInfoToast({
			message: m.hiddenModsPref_reset_toast({ count })
		});
	}
</script>

<div class="flex items-center">
	<Label>{m.hiddenModsPref_title()}</Label>

	<Info>
		{m.hiddenModsPref_content()}
	</Info>

	<button
		class="group bg-primary-900 enabled:hover:border-primary-500 text-primary-300 relative flex grow items-center gap-2 truncate rounded-lg border border-transparent px-3 py-1 disabled:cursor-not-allowed disabled:opacity-70"
		disabled={!hasHiddenMods}
		onclick={() => (dialogOpen = true)}
	>
		<Icon icon="mdi:eye-off" />

		<div class="truncate">
			{#if hasHiddenMods}
				{m.hiddenModsPref_button_content({ count: hiddenMods.length })}
			{:else}
				{m.hiddenModsPref_button_content_empty()}
			{/if}
		</div>
	</button>

	{#if hasHiddenMods}
		<ResetButton onclick={() => onResetClicked()} class="ml-1" />
	{/if}
</div>

<Dialog bind:open={dialogOpen} title="Hidden Mods">
	<ModCardList class="my-2 max-h-[50vh] overflow-y-auto" mods={hiddenMods} showVersion={false}>
		{#snippet cardChildren({ mod })}
			<br />
			<IconButton
				icon="mdi:eye"
				color="primary"
				class="block"
				label="Unhide"
				onclick={() => unHideMod(mod)}
			/>
		{/snippet}
	</ModCardList>
</Dialog>
