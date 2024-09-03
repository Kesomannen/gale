<script lang="ts">
	import { refreshProfiles } from '$lib/stores';
	import { invokeCommand } from '$lib/invoke';
	import BigButton from '$lib/components/BigButton.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import { t } from '$i18n';

	export let open = false;

	let name = '';

	async function createProfile() {
		if (name.length === 0) return;

		await invokeCommand('create_profile', { name });
		refreshProfiles();
		name = '';
		open = false;
	}
</script>

<ConfirmPopup title="{t('Create new profile')}" bind:open>
	<p class="mb-1">{t('Create new profile description')}</p>
	<InputField
		placeholder="{t('Enter profile name')}"
		size="lg"
		class="w-full"
		on:submit={createProfile}
		bind:value={name}
	/>
	<svelte:fragment slot="buttons">
		<BigButton disabled={name.length === 0} on:click={createProfile}>{t("Create")}</BigButton>
	</svelte:fragment>
</ConfirmPopup>
