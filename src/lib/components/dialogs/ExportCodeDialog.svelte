<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import * as api from '$lib/api';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '../ui/IconButton.svelte';
	import { pushInfoToast } from '$lib/toast';
	import { Backend, type ExportCodeResult } from '$lib/types';
	import InfoBox from '$lib/components/ui/InfoBox.svelte';
	import ExportFilesDialog from './ExportFilesDialog.svelte';
	import Button from '../ui/Button.svelte';

	let isOpen = $state(false);
	let exporting = $state(false);
	let result = $state<ExportCodeResult | null>(null);

	export async function open() {
		try {
			result = null;
			isOpen = true;

			if (!exporting) {
				exporting = true;
				result = await api.profile.export.code();
			}
		} catch (error) {
			isOpen = false;
		} finally {
			exporting = false;
		}
	}

	async function copyCode() {
		if (!result || result.type === 'tooLarge') return;

		await writeText(result.code);

		pushInfoToast({
			message: m.exportCodeDialog_copyCode_message()
		});
	}
</script>

<Dialog title={m.exportCodeDialog_title()} bind:open={isOpen}>
	<div class="text-primary-600 dark:text-primary-300 mt-1 space-y-1">
		{#if result}
			{#if result.type === 'tooLarge'}
				<ExportFilesDialog open title={m.exportCodeDialog_sizeLimit_title()}>
					{#snippet description()}
						{m.exportCodeDialog_sizeLimit_content()}
					{/snippet}

					{#snippet buttons()}
						<Button
							color="primary"
							onclick={() => {
								isOpen = false;
								result = null;
							}}>{m.exportCodeDialog_sizeLimit_button_cancel()}</Button
						>
						<Button icon="mdi:refresh" color="accent" onclick={open}
							>{m.exportCodeDialog_sizeLimit_button_retry()}</Button
						>
					{/snippet}
				</ExportFilesDialog>
			{:else}
				<div>
					{m.exportCodeDialog_done()}
				</div>

				<div>
					<button
						class="text-primary-700 dark:bg-primary-900 dark:text-primary-300 bg-primary-100 rounded-md px-4 py-1 font-mono text-lg"
						onclick={copyCode}
					>
						{result.code}
					</button>

					<IconButton
						icon="mdi:content-copy"
						label={m.exportCodeDialog_copyCode_label()}
						onclick={copyCode}
					/>
				</div>

				{#if result.backend !== Backend.Thunderstore}
					<InfoBox type="info">
						{m.exportCodeDialog_galeExclusive()}
					</InfoBox>
				{/if}
			{/if}
		{:else}
			<div class="flex items-center gap-1">
				<Spinner class="text-lg" />
				<span>
					{m.exportCodeDialog_loading({
						name: profiles.active?.name ?? m.exportCodeDialog_content_unknown()
					})}
				</span>
			</div>
		{/if}
	</div>
</Dialog>
