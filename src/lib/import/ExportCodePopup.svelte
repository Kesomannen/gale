<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import { Dialog } from 'bits-ui';
	import { invoke } from '$lib/invoke';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { activeProfile } from '$lib/stores.svelte';
	import Spinner from '$lib/components/Spinner.svelte';

	let isOpen = $state(false);
	let codePromise: Promise<string> | null = $state(null);

	export async function open() {
		codePromise = invoke('export_code');
		isOpen = true;

		try {
			let code = await codePromise;
			await writeText(code);
		} catch (e) {
			isOpen = false;
		}
	}
</script>

<Popup title="Export as code" bind:open={isOpen}>
	<Dialog.Description class="flex-center text-primary-400 mb-1 flex">
		{#await codePromise}
			<Spinner class="text-lg" />
			Exporting {$activeProfile?.name} as code...
		{:then}
			Export complete! The code has been copied to your clipboard:
		{/await}
	</Dialog.Description>

	{#await codePromise then code}
		<code class="bg-primary-900 text-primary-400 rounded-sm px-3 py-1 text-lg">
			{code}
		</code>
	{/await}
</Popup>
