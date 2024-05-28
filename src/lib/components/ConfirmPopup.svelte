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
	<Dialog.Description class="text-slate-300">
		<slot>
			{description}
		</slot>
	</Dialog.Description>

	<Dialog.Close class="flex w-full justify-end mt-3 mr-0.5 gap-2">
		<BigButton color="gray" onClick={onCancel}>Cancel</BigButton>
		<slot name="buttons" />
	</Dialog.Close>
</Popup>
