<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import * as api from '$lib/api';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '../ui/IconButton.svelte';
	import { pushInfoToast } from '$lib/toast';
	import { Backend, type ExportCode } from '$lib/types';

	let isOpen = $state(false);
	let code = $state<ExportCode | null>(null);

	export async function open() {
		try {
			code = null;
			isOpen = true;
			code = await api.profile.export.code();
		} catch (e) {
			isOpen = false;
		}
	}

	async function copyCode() {
		if (!code) return;

		await writeText(code.code);

		pushInfoToast({
			message: m.exportCodeDialog_copyCode_message()
		});
	}
</script>

<Dialog title={m.exportCodeDialog_title()} bind:open={isOpen}>
	<div class="text-primary-300 mt-1 space-y-1">
		{#if code}
			<div>
				{m.exportCodeDialog_done()}
			</div>

			<div>
				<button
					class="bg-primary-900 text-primary-300 rounded-md px-4 py-1 font-mono text-lg"
					onclick={copyCode}
				>
					{code.code}
				</button>

				<IconButton
					icon="mdi:content-copy"
					label={m.exportCodeDialog_copyCode_label()}
					onclick={copyCode}
				/>
			</div>

			{#if code.backend !== Backend.Thunderstore}
				<div>
					{m.exportCodeDialog_galeExclusive()}
				</div>
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
