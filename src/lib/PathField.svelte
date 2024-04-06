<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';

	export let label: string;
	export let onClick: () => void;
	export let value: string | null;
	export let icon: string = 'mdi:folder';
</script>

<div class="flex items-center">
	<div class="text-slate-300 text-md w-52 truncate">{label}</div>

	<Button.Root
		class="flex flex-grow px-3 py-1.5 items-center text-right rounded-md group bg-gray-900 truncate
            border border-gray-500 border-opacity-0 hover:border-opacity-100"
		on:click={onClick}
	>
		<div class="mr-2 rounded">
			<Icon {icon} class="align-middle text-slate-400 group-hover:text-slate-200" />
		</div>

		<div
			class="font-light text-slate-400 group-hover:text-slate-200 truncate text-sm"
			style="direction: rtl;"
		>
			{value ?? 'Not set'}
		</div>
	</Button.Root>
</div>
