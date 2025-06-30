<script lang="ts">
	import { Menubar } from 'bits-ui';

	type Props = {
		text: string;
		key?: string | undefined;
		onclick: () => void;
	};

	let { text, key = undefined, onclick }: Props = $props();
</script>

<Menubar.Item
	class="group hover:bg-primary-700 mx-1 flex cursor-default items-center justify-between rounded py-1 pl-4 text-left {key
		? 'pr-4'
		: 'pr-10'}"
	{onclick}
>
	<span class="text-primary-400 group-hover:text-primary-300">{text}</span>

	{#if key}
		<span class="text-primary-400 group-hover:text-primary-300 pl-6 font-mono text-sm">{key}</span>
	{/if}
</Menubar.Item>
