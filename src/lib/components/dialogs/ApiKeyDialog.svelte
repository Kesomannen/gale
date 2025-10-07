<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Link from '$lib/components/ui/Link.svelte';
	import * as api from '$lib/api';
	import { apiKeyDialog } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';

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

<ConfirmDialog title={m.apiKeyDialog_title()} bind:open={apiKeyDialog.open}>
	<p>{m.apiKeyDialog_content_1()}</p>

	<p class="mt-2">
		{m.apiKeyDialog_content_2()}
	</p>

	<p class="mt-2 mb-1">
		{m.apiKeyDialog_content_3()}<b>{m.apiKeyDialog_content_4()}</b>{m.apiKeyDialog_content_5()}
	</p>

	<InputField
		placeholder={m.apiKeyDialog_placeholder()}
		class="w-full"
		onsubmit={submit}
		bind:value={token}
	/>

	<Link
		class="mt-2 block max-w-max text-sm"
		href="https://github.com/Kesomannen/gale/wiki/Getting-a-Thunderstore-API-token"
		>
		{m.apiKeyDialog_link()}
	</Link>

	{#snippet buttons()}
		<Button icon="mdi:exchange" color="accent" onclick={submit} {loading}>{m.apiKeyDialog_button()}</Button>
	{/snippet}
</ConfirmDialog>
