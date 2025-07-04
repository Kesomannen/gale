<script lang="ts">
	import { Dialog } from 'bits-ui';
	import Popup from './Popup.svelte';
	import Button from './Button.svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		title?: string;
		description?: string;
		open?: boolean;
		onCancel?: () => void;
		children?: Snippet;
		buttons?: Snippet;
	};

	let {
		title = '',
		description = '',
		open = $bindable(false),
		onCancel = () => {},
		children,
		buttons
	}: Props = $props();
</script>

<Popup {title} onclose={onCancel} bind:open>
	<Dialog.Description class="text-primary-300">
		{#if children}{@render children()}{:else}
			{description}
		{/if}
	</Dialog.Description>

	<div class="mt-3 ml-auto flex justify-end gap-2 overflow-hidden">
		<Button
			color="primary"
			onclick={() => {
				onCancel();
				open = false;
			}}>Cancel</Button
		>
		{@render buttons?.()}
	</div>
</Popup>
