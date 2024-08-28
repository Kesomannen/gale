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

	import { t } from '$i18n';

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

<ConfirmPopup title="{t["Set thunderstore token"]}" bind:open={$apiKeyPopupOpen}>
	<p>
		{t["Set thunderstore token description 1"]}
	</p>

	<p class="mt-1 mb-2">
		{@html t["Set thunderstore token description 2"]}
	</p>

	<InputField placeholder="{t["Enter API token"]}" class="w-full" on:submit={submit} bind:value={token} />

	<details>
		<summary class="text-sm text-slate-400 mt-1 cursor-pointer"
			>{t["Unsure thunderstore token"]}</summary
		>
		<ol class="mt-1 ml-1 flex flex-col gap-1">
			<li>
				{t["Unsure thunderstore token description 1"]}
				<Link href="https://thunderstore.io/">thunderstore.io</Link>
				{t["Unsure thunderstore token description 2"]}
			</li>

			<li>
				{t["Unsure thunderstore token description 3"]}
				<Link href="https://thunderstore.io/settings/teams/">Teams</Link>
				{t["Unsure thunderstore token description 4"]}
			</li>

			<li>
				{t["Unsure thunderstore token description 5"]}
			</li>

			<li>
				{@html t["Unsure thunderstore token description 6"]}
			</li>

			<li>
				{@html t["Unsure thunderstore token description 7"]}
			</li>

			<li>
				{t["Unsure thunderstore token description 8"]}
			</li>
		</ol>

		<b>{t["Unsure thunderstore token description 9"]}</b>
	</details>

	<svelte:fragment slot="buttons">
		<BigButton color="green" fontWeight="medium" on:click={submit}>{t["Submit"]}</BigButton>
	</svelte:fragment>
</ConfirmPopup>
