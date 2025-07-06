<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import * as api from '$lib/api';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import profiles from '$lib/state/profile.svelte';

	let isOpen = $state(false);
	let codePromise: Promise<string> | null = $state(null);

	export async function open() {
		codePromise = api.profile.export.code();
		isOpen = true;

		try {
			let code = await codePromise;
			await writeText(code);
		} catch (e) {
			isOpen = false;
		}
	}
</script>

<Dialog title="Export as code" bind:open={isOpen}>
	<p class="flex-center text-primary-400 mb-1 flex">
		{#await codePromise}
			<Spinner class="text-lg" />
			Exporting {profiles.active?.name} as code...
		{:then}
			Export complete! The code has been copied to your clipboard:
		{/await}
	</p>

	{#await codePromise then code}
		<code class="bg-primary-900 text-primary-400 rounded-sm px-3 py-1 text-lg">
			{code}
		</code>
	{/await}
</Dialog>
