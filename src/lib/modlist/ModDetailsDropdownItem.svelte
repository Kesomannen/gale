<script lang="ts">
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';

  export let label: string;
  export let icon: string;
  export let onClick: () => void;
</script>

<DropdownMenu.Item
	class="flex items-center pl-3 pr-5 py-1 truncate text-slate-300 hover:text-slate-100 text-left rounded-md hover:bg-gray-600 cursor-default"
	on:click={onClick}
>
	<Icon class="text-lg mr-1.5" {icon} />
	{label}
</DropdownMenu.Item>
