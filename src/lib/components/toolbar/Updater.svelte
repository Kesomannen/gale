<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import Icon from '@iconify/svelte';
	import { message } from '@tauri-apps/plugin-dialog';
	import { platform } from '@tauri-apps/plugin-os';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { Dialog } from 'bits-ui';
	import { onMount } from 'svelte';
	import { pushToast } from '$lib/toast';
	import updates from '$lib/state/update.svelte';
	import { m } from '$lib/paraglide/messages';

	let dialogOpen = $state(false);
	let loading = $state(false);

	onMount(() => {
		updates.refresh();
	});

	async function installUpdate() {
		if (!updates.next) return;

		try {
			await updates.next.downloadAndInstall();
		} catch (error) {
			let message: string;
			if (typeof error === 'string') {
				message = error;
			} else if (error instanceof Error) {
				message = error.message;
			} else {
				message = m.updater_installUpdate_message_unknown();
			}

			pushToast({
				type: 'error',
				name: m.updater_installUpdate_message_name(),
				message
			});
		}

		updates.next = null;
	}

	async function update() {
		dialogOpen = false;
		loading = true;
		await installUpdate();
		loading = false;

		if (platform() !== 'windows') {
			// on other platforms installUpdate() doesn't relaunch the app itself
			await message(m.updater_update_message());
			await relaunch();
		}
	}
</script>

{#if updates.next}
	<button
		class="bg-accent-700 enabled:hover:bg-accent-600 text-primary-100 mx-2 my-auto ml-auto flex items-center gap-1 rounded-md px-2.5 py-1 text-sm font-bold"
		disabled={loading}
		onclick={() => (dialogOpen = true)}
	>
		{#if loading}
			<Icon icon="mdi:loading" class="animate-spin" />
		{:else}
			<Icon icon="mdi:arrow-up-circle" />
		{/if}
		<div class="truncate text-sm">
			{m[`updater_content_${loading ? 'downloading' : 'available'}`]()}
		</div>
	</button>
{/if}

<ConfirmDialog title={m.updater_confirmDialog_title()} bind:open={dialogOpen}>
	<Dialog.Description class="text-primary-300">
		<p>
			{#if updates.next}
				{m.updater_confirmDialog_content_next({
					next: updates.next.version,
					current: updates.next.currentVersion
				})}
			{:else}
				{m.updater_confirmDialog_content_available()}
			{/if}

			{m.updater_confirmDialog_content()}
		</p>
	</Dialog.Description>

	{#snippet buttons()}
		<Button color="accent" onclick={update}>{m.updater_confirmDialog_button()}</Button>
	{/snippet}
</ConfirmDialog>
