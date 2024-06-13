<script context="module" lang="ts">
	export const apiKeyPopupOpen = writable(false);
</script>

<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import Link from '$lib/components/Link.svelte';
	import { invokeCommand } from '$lib/invoke';
	import { Button } from 'bits-ui';

	import { writable } from 'svelte/store';

	let token: string;

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

	<p class="mt-1 mb-2">
		Once set, you <b>will not</b> be able to view the token again.
	</p>

	<InputField placeholder="Enter API token..." onSubmit={submit} bind:value={token} />

	<details>
		<summary class="text-sm text-slate-400 mt-1 cursor-pointer"
			>Unsure how to get your API token?</summary
		>
		<ol class="mt-1 ml-1 flex flex-col gap-1">
			<li>
				1. Login to the <Link href="https://thunderstore.io/">thunderstore website</Link>.
			</li>

			<li>
				2. Go to <Link href="https://thunderstore.io/settings/teams/">Teams</Link>.
			</li>

			<li>
				3. If you don't have one already, create a new team. The name should be your own username.
			</li>

			<li>
				4. Select a team and go to <code>Service Accounts</code> on the left sidebar.
			</li>

			<li>
				5. Click <code>Add service account</code> and choose an appropriate nickname, for example "gale".
			</li>

			<li>
				6. Once you submit, the API token will be displayed. Make sure you copy and paste it here,
				since you won't be able to see it once you navigate away from the page.
			</li>

			<b
				>Do not share the token with anyone else, as it gives power to update, publish or delete
				packages in your name!</b
			>
		</ol>
	</details>

	<svelte:fragment slot="buttons">
		<BigButton color="green" on:click={submit}>Submit</BigButton>
	</svelte:fragment>
</ConfirmPopup>
