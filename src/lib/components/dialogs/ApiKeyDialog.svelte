<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Link from '$lib/components/ui/Link.svelte';
	import * as api from '$lib/api';
	import { apiKeyDialog } from '$lib/state/misc.svelte';

	let token: string = $state('');
	let loading = $state(false);

	async function submit() {
		loading = true;

		try {
			if (token.length == 0) {
				await api.thunderstore.clearToken();
			} else {
				await api.thunderstore.setToken(token);
				token = '';
			}
		} finally {
			loading = false;
		}

		apiKeyDialog.open = false;
	}
</script>

<ConfirmDialog title="Set thunderstore API token" bind:open={apiKeyDialog.open}>
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

	<Link
		class="mt-2 block max-w-max text-sm"
		href="https://github.com/Kesomannen/gale/wiki/Getting-a-Thunderstore-API-token"
		>Unsure how to get your API token?</Link
	>

	{#snippet buttons()}
		<Button icon="mdi:exchange" color="accent" onclick={submit} {loading}>Submit</Button>
	{/snippet}
</ConfirmDialog>
