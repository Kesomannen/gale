<script lang="ts">
	import { Button } from 'bits-ui';

	export let disabled: boolean = false;
	export let color: string = 'accent';
	export let fontWeight: string = 'normal';
	export { className as class };

	let className = '';
</script>

<Button.Root
	class="{className} inline-flex items-center overflow-hidden rounded-lg font-{fontWeight} px-4 py-2 text-white bg-{color}-700 enabled:hover:bg-{color}-600 disabled:opactiy-70 text-nowrap disabled:cursor-not-allowed disabled:bg-gray-700 disabled:text-gray-400"
	on:click
	bind:disabled
>
	<slot />
</Button.Root>
