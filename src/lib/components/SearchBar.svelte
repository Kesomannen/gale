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
	class="w-full rounded-lg px-10 py-1.5 bg-slate-{brightness} valid:focus:ring-accent-400 truncate border border-transparent text-slate-300 placeholder-slate-400 invalid:ring-red-500 hover:border-slate-500 focus:ring-2 focus:outline-hidden text-{size}"
	bind:value
	on:keydown
	{placeholder}
/>
<Icon class="absolute top-2 left-2.5 text-2xl text-slate-400" icon="mdi:magnify" />
<Button.Root
	class="absolute top-[0.4rem] right-1.5 rounded-md p-1 text-xl text-slate-400 hover:bg-slate-800 hover:text-slate-300"
	on:click={() => (value = '')}
>
	<Icon icon="mdi:close" />
</Button.Root>
