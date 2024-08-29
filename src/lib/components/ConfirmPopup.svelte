<script lang="ts">
	import { Dialog } from 'bits-ui';
	import Popup from './Popup.svelte';
	import BigButton from './BigButton.svelte';
	import { get } from 'svelte/store';
	import { t } from '$i18n';

	export let title = '';
	export let description = '';
	export let open = false;

	export let onCancel: () => void = () => {};
</script>

<Popup {title} onClose={onCancel} bind:open>
	<Dialog.Description class="text-slate-300">
		<slot>
			{description}
		</slot>
	</Dialog.Description>

	<Dialog.Close class="flex ml-auto justify-end mt-3 gap-2">
		<BigButton color="gray" on:click={onCancel}>{get(t)["Cancel"]}</BigButton>
		<slot name="buttons" />
	</Dialog.Close>
</Popup>
