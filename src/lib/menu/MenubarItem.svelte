<script lang="ts">
	import { Menubar } from 'bits-ui';

	export let text: string;
	export let key: string | undefined = undefined;
</script>

<Menubar.Item
	class="group mx-1 flex cursor-default items-center justify-between rounded-md py-1 pl-4 text-left hover:bg-gray-700 {key
		? 'pr-4'
		: 'pr-10'}"
	on:click
>
	<span class="text-slate-400 group-hover:text-slate-300">{text}</span>

	{#if key}
		<span class="pl-6 font-mono text-sm text-slate-400 group-hover:text-slate-300">{key}</span>
	{/if}
</Menubar.Item>
