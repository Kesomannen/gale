<script lang="ts">
	import { Dialog } from 'bits-ui';
	import Popup from './Popup.svelte';
	import BigButton from './BigButton.svelte';

	export let title = '';
	export let description = '';
	export let open = false;

	export let onCancel: () => void = () => {};
</script>

<Popup {title} onClose={onCancel} bind:open>
	<Dialog.Description class="text-primary-300">
		<slot>
			{description}
		</slot>
	</Dialog.Description>

	<div class="mt-3 ml-auto flex justify-end gap-2 overflow-hidden">
		<BigButton
			color="primary"
			on:click={() => {
				onCancel();
				open = false;
			}}>Cancel</BigButton
		>
		<slot name="buttons" />
	</div>
</Popup>
