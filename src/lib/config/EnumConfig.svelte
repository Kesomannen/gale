<script lang="ts">
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import ResetConfigButton from './ResetConfigButton.svelte';
	import { setTaggedConfig } from '$lib/invoke';
	import Dropdown from '$lib/components/Dropdown.svelte';

	export let entryId: ConfigEntryId;

	let content = entryId.entry.value.content as { index: number; options: string[] };
	let selected = content.options[content.index];

	function onReset(newValue: ConfigValue) {
		content = newValue.content as { index: number; options: string[] };
	}

	function onSelectChange(value: string) {
		let index = content.options.indexOf(value);
		setTaggedConfig(entryId, {
			type: 'enum',
			content: {
				index,
				options: content.options
			}
		});
	}
</script>

<Dropdown
	items={content.options}
	class="flex-grow overflow-hidden"
	bind:selected
	onSelectedChangeSingle={onSelectChange}
	getLabel={(value) => value}
	size='sm'
/>
<ResetConfigButton {entryId} {onReset} />
