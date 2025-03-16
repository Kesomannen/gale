<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';

	export let value: string;
	export let placeholder: string;
	export let brightness: number = 900;
	export let size: 'sm' | 'base' | 'lg' = 'base';
</script>

<input
	type="text"
	class="w-full rounded-lg px-10 py-1.5 bg-primary-{brightness} valid:focus:ring-accent-400 text-primary-300 placeholder-primary-400 hover:border-primary-500 truncate border border-transparent invalid:ring-red-500 focus:ring-2 focus:outline-hidden text-{size}"
	bind:value
	on:keydown
	{placeholder}
/>
<Icon class="text-primary-400 absolute top-2 left-2.5 text-2xl" icon="mdi:magnify" />
<Button.Root
	class="text-primary-400 hover:bg-primary-800 hover:text-primary-300 absolute top-[0.4rem] right-1.5 rounded-md p-1 text-xl"
	on:click={() => (value = '')}
>
	<Icon icon="mdi:close" />
</Button.Root>
