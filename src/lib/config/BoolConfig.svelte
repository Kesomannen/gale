<script lang="ts">
	import { setConfig } from '$lib/invoke';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import { Checkbox } from 'bits-ui';
	import ResetConfigButton from './ResetConfigButton.svelte';
	import Icon from '@iconify/svelte';

	export let entryId: ConfigEntryId;

	let content = entryId.entry.value.content as boolean;

	function onReset(newValue: ConfigValue) {
		content = newValue.content as boolean;
	}

	function onCheckedChange(newValue: boolean | 'indeterminate') {
		if (newValue === 'indeterminate') return;

		content = newValue;
		setConfig(entryId, { type: 'boolean', content });
	}
</script>

<div class="flex items-center flex-grow">
	<Checkbox.Root bind:checked={content} {onCheckedChange}>
		<Checkbox.Indicator
			class="rounded-md w-5 h-5 p-1 
            bg-{content ? 'green-700' : 'gray-800'}
            hover:bg-{content ? 'green-600' : 'gray-700'}
            {content ? '' : 'border border-gray-500'}"
		>
			{#if content}
				<Icon class="text-white w-full h-full font-bold" icon="mdi:check" />
			{/if}
		</Checkbox.Indicator>
	</Checkbox.Root>
</div>
<ResetConfigButton {entryId} {onReset} />
