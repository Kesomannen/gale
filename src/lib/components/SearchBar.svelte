<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';

	export let value: string;
	export let placeholder: string;
  export let brightness: number = 900;
</script>

<input
	type="text"
	class="py-1.5 px-10 w-full rounded-lg bg-gray-{brightness} truncate text-slate-300 placeholder-slate-400
          border border-gray-500 border-opacity-0 hover:border-opacity-100
		  valid:focus:ring-green-400 invalid:ring-red-500 focus:ring-2 focus:outline-none"
	bind:value
	{placeholder}
/>
<Icon class="absolute left-2.5 top-2 text-slate-500 text-2xl" icon="mdi:magnify" />
<Button.Root
	class="absolute right-1.5 top-[0.4rem] rounded-md text-xl text-slate-500 p-1
          hover:bg-gray-800 hover:text-slate-400"
	on:click={() => (value = '')}
>
	<Icon icon="mdi:close" />
</Button.Root>
