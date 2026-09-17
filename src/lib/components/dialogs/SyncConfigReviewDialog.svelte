<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import * as api from '$lib/api';
	import type { PendingSyncConfigUpdate } from '$lib/types';
	import { pushInfoToast } from '$lib/toast';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { SvelteSet } from 'svelte/reactivity';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		open: boolean;
		updates: PendingSyncConfigUpdate[];
		onChanged: () => void | Promise<void>;
	};

	let { open = $bindable(), updates, onChanged }: Props = $props();

	let selected: SvelteSet<string> = $state(new SvelteSet());
	let loading = $state(false);

	let reasonLabel = $derived({
		modifiedLocally: m.syncConfigReviewDialog_reason_modifiedLocally(),
		deletedLocally: m.syncConfigReviewDialog_reason_deletedLocally()
	});

	$effect(() => {
		if (open) {
			selected = new SvelteSet();
		}
	});

	async function applySelected() {
		let deletedCount = [...selected].filter(
			(path) => updates.find((update) => update.path === path)?.reason === 'deletedLocally'
		).length;

		if (deletedCount > 0) {
			let confirmed = await confirm(
				m.syncConfigReviewDialog_restoreConfirm({ count: deletedCount })
			);
			if (!confirmed) return;
		}

		loading = true;
		try {
			await api.profile.sync.applyConfig([...selected]);
			pushInfoToast({ message: m.syncConfigReviewDialog_applyMessage() });
			selected = new SvelteSet();
			await onChanged();
		} finally {
			loading = false;
		}
	}

	async function declineSelected() {
		loading = true;
		try {
			await api.profile.sync.declineConfig([...selected]);
			pushInfoToast({ message: m.syncConfigReviewDialog_declineMessage() });
			selected = new SvelteSet();
			await onChanged();
		} finally {
			loading = false;
		}
	}
</script>

<Dialog bind:open title={m.syncConfigReviewDialog_title()}>
	{#if updates.length === 0}
		<div class="text-primary-600 dark:text-primary-300 my-4 text-center">
			{m.syncConfigReviewDialog_empty()}
		</div>
	{:else}
		<div class="mt-3 flex max-h-60 flex-col gap-1 overflow-y-auto pr-1">
			{#each updates as update (update.path)}
				<div
					class="dark:bg-primary-900 bg-primary-100 flex items-center gap-2.5 rounded-md px-3 py-1.5"
				>
					<Checkbox
						checked={selected.has(update.path)}
						onCheckedChange={(checked) => {
							if (checked) {
								selected.add(update.path);
							} else {
								selected.delete(update.path);
							}
						}}
					/>
					<span class="text-primary-800 dark:text-primary-100 truncate" title={update.path}>
						{update.path}
					</span>
					<span class="text-primary-500 dark:text-primary-400 shrink-0 text-sm">
						{reasonLabel[update.reason]}
					</span>
					{#if update.declined}
						<span
							class="bg-primary-300 text-primary-600 dark:bg-primary-700 dark:text-primary-300 ml-auto shrink-0 rounded px-2 py-0.5 text-xs font-medium"
						>
							{m.syncConfigReviewDialog_declined()}
						</span>
					{/if}
				</div>
			{/each}
		</div>

		<div class="mt-4 flex justify-end gap-2">
			<Button
				color="primary"
				onclick={declineSelected}
				{loading}
				disabled={selected.size === 0}
				icon="mdi:close"
			>
				{m.syncConfigReviewDialog_decline()}
			</Button>
			<Button onclick={applySelected} {loading} disabled={selected.size === 0} icon="mdi:check">
				{m.syncConfigReviewDialog_apply()}
			</Button>
		</div>
	{/if}
</Dialog>
