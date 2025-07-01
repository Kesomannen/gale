<script module lang="ts">
	export const apiKeyPopupOpen = writable(false);
</script>

<script lang="ts">
	import BigButton from '$lib/components/Button.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import Link from '$lib/components/Link.svelte';
	import { invokeCommand } from '$lib/invoke';

	import { writable } from 'svelte/store';

	let token: string = $state('');

	async function submit() {
		if (token.length == 0) {
			await invokeCommand('clear_thunderstore_token');
		} else {
			await invokeCommand('set_thunderstore_token', { token });
			token = '';
		}

		$apiKeyPopupOpen = false;
	}
</script>

<ConfirmPopup title="Set thunderstore API token" bind:open={$apiKeyPopupOpen}>
	<p>
		Enter your Thunderstore API token below, or leave blank to clear the current one. This token is
		used to publish modpacks to Thunderstore, and will be stored securely on your computer.
	</p>

	<p class="mt-2 mb-3">
		Once set, you will <b>not</b> be able to view the token again.
	</p>

	<InputField
		placeholder="Enter API token..."
		class="w-full"
		onsubmit={submit}
		bind:value={token}
	/>

	<Link class="text-sm" href="https://example.com">Unsure how to get your API token?</Link>

	{#snippet buttons()}
		<BigButton color="accent" fontWeight="medium" onclick={submit}>Submit</BigButton>
	{/snippet}
</ConfirmPopup>
