<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Link from '$lib/components/ui/Link.svelte';
	import * as api from '$lib/api';
	import { apiKeyDialog } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import { Backend } from '$lib/types';

	let token: string = $state('');
	let loading = $state(false);

	let helpUrl = $derived(
		apiKeyDialog.backend === Backend.Thunderstore
			? 'https://github.com/Kesomannen/gale/wiki/Getting-a-Thunderstore-API-token'
			: 'https://mods.valtools.org/faq#api-token'
	);

	async function submit() {
		loading = true;

		try {
			if (token.length == 0) {
				await api.thunderstore.clearToken(apiKeyDialog.backend);
			} else {
				await api.thunderstore.setToken(apiKeyDialog.backend, token);
				token = '';
			}
		} finally {
			loading = false;
		}

		apiKeyDialog.open = false;
	}
</script>

<ConfirmDialog
	title={m.apiKeyDialog_title({ backend: apiKeyDialog.backend })}
	bind:open={apiKeyDialog.open}
>
	<p>{m.apiKeyDialog_content_1({ backend: apiKeyDialog.backend })}</p>

	<p class="mt-2">
		{m.apiKeyDialog_content_2({ backend: apiKeyDialog.backend })}
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

	<Link class="mt-2 block max-w-max text-sm" href={helpUrl}>
		{m.apiKeyDialog_link()}
	</Link>

	{#snippet buttons()}
		<Button icon="mdi:exchange" color="accent" onclick={submit} {loading}
			>{m.apiKeyDialog_button()}</Button
		>
	{/snippet}
</ConfirmDialog>
