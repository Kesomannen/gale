<script lang="ts">
	import { Menubar } from 'bits-ui';

	export let text: string;
	export let key: string | undefined = undefined;
</script>

<Menubar.Item
	class="group hover:bg-primary-700 mx-1 flex cursor-default items-center justify-between rounded py-1 pl-4 text-left {key
		? 'pr-4'
		: 'pr-10'}"
	on:click
>
	<span class="text-primary-400 group-hover:text-primary-300">{text}</span>

	{#if key}
		<span class="text-primary-400 group-hover:text-primary-300 pl-6 font-mono text-sm">{key}</span>
	{/if}
</Menubar.Item>
