<script lang="ts">
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';

	export let label: string;
	export let icon: string;
	export let onClick: () => void;
</script>

<DropdownMenu.Item
	class="flex cursor-default items-center truncate rounded-md py-1 pl-3 pr-5 text-left text-slate-300 hover:bg-gray-600 hover:text-slate-100"
	on:click={onClick}
>
	<Icon class="mr-1.5 text-lg" {icon} />
	{label}
</DropdownMenu.Item>
