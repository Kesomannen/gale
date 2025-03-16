<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import Label from './Label.svelte';
	import Info from './Info.svelte';

	export let label: string | null = null;
	export let value: string | null;
	export let icon: string = 'mdi:folder';

	$: hasValue = value && value.length > 0;
</script>

<div class="relative flex items-center">
	{#if label !== null}
		<Label>
			{label}
		</Label>
	{/if}

	<Info>
		<slot />
	</Info>

	<Button.Root
		class="group bg-primary-900 hover:border-primary-500 flex grow basis-0 items-center truncate rounded-lg border border-transparent px-3 py-1 text-right"
		on:click
	>
		<div class="mr-2 rounded-sm">
			<Icon {icon} class="text-primary-300 align-middle" />
		</div>

		<div class="text-primary-300 truncate" style="direction: rtl;">
			&#x200E;
			{hasValue ? value : 'Not set'}
		</div>

		<slot name="field" />
	</Button.Root>
</div>
