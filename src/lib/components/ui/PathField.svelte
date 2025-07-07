<script lang="ts">
	import Icon from '@iconify/svelte';
	import Label from './Label.svelte';
	import Info from './Info.svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		label?: string | null;
		value: string | null;
		icon?: string;
		onclick?: () => void;
		children?: Snippet;
		field?: Snippet;
	};

	let {
		label = null,
		value = $bindable(),
		icon = 'mdi:folder',
		onclick,
		children,
		field
	}: Props = $props();

	let hasValue = $derived(value && value.length > 0);
</script>

<div class="relative flex items-center">
	{#if label !== null}
		<Label>
			{label}
		</Label>
	{/if}

	<Info>
		{@render children?.()}
	</Info>

	<button
		class="group bg-primary-900 hover:border-primary-500 flex grow basis-0 items-center truncate rounded-lg border border-transparent py-1 pr-1 pl-3 text-right"
		{onclick}
	>
		<div class="mr-2 rounded-sm">
			<Icon {icon} class="text-primary-300 align-middle" />
		</div>

		<div class="text-primary-300 truncate" style="direction: rtl;">
			&#x200E;
			{hasValue ? value : 'Not set'}
		</div>

		{@render field?.()}
	</button>
</div>
