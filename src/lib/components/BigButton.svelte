<script lang="ts">
	import { Button } from 'bits-ui';

	export let disabled: boolean = false;
	export let color: string = 'green';
	export let fontWeight: string = 'normal';
</script>

<Button.Root
	class="inline-flex items-center rounded-lg font-{fontWeight} px-4 py-2 text-slate-100 bg-{color}-700 hover:bg-{color}-600 text-nowrap disabled:cursor-not-allowed disabled:bg-gray-600/80 disabled:text-gray-200/80 disabled:hover:bg-gray-600/80"
	on:click
	bind:disabled
>
	<slot />
</Button.Root>
