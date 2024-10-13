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
	class="w-full rounded-lg px-10 py-1.5 bg-gray-{brightness} truncate border border-gray-500 border-opacity-0 text-slate-300 placeholder-slate-400 invalid:ring-red-500 hover:border-opacity-100 focus:outline-none focus:ring-2 valid:focus:ring-green-400 text-{size}"
	bind:value
	on:keydown
	{placeholder}
/>
<Icon class="absolute left-2.5 top-2 text-2xl text-slate-500" icon="mdi:magnify" />
<Button.Root
	class="absolute right-1.5 top-[0.4rem] rounded-md p-1 text-xl text-slate-500 hover:bg-gray-800 hover:text-slate-400"
	on:click={() => (value = '')}
>
	<Icon icon="mdi:close" />
</Button.Root>
