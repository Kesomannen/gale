<script module lang="ts">
	export const apiKeyDialogOpen = writable(false);
</script>

<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Link from '$lib/components/ui/Link.svelte';
	import * as api from '$lib/api';

	import { writable } from 'svelte/store';

	let token: string = $state('');

	async function submit() {
		if (token.length == 0) {
			await api.thunderstore.clearToken();
		} else {
			await api.thunderstore.setToken(token);
			token = '';
		}

		$apiKeyDialogOpen = false;
	}
</script>

<ConfirmDialog title="Set thunderstore API token" bind:open={$apiKeyDialogOpen}>
	<p>Enter your Thunderstore API token below, or leave blank to clear the current one.</p>

	<p class="mt-2">
		This token is used to publish modpacks to Thunderstore, and will be stored securely on your
		computer.
	</p>

	<p class="mt-2 mb-1">
		Once set, you will <b>not</b> be able to view the token again.
	</p>

	<InputField
		placeholder="Enter API token..."
		class="w-full"
		onsubmit={submit}
		bind:value={token}
	/>

	<Link class="mt-2 block text-sm" href="https://example.com"
		>Unsure how to get your API token?</Link
	>

	{#snippet buttons()}
		<Button color="accent" onclick={submit}>Submit</Button>
	{/snippet}
</ConfirmDialog>
