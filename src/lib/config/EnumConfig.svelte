<script lang="ts">
	import type { ConfigEntryId, ConfigValue } from '$lib/types';
	import ResetConfigButton from './ResetConfigButton.svelte';
	import { setConfigEntry } from '$lib/config';
	import Select from '$lib/components/Select.svelte';
	import { selectItems } from '$lib/util';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
	};

	let { entryId, locked }: Props = $props();

	let content = $state(entryId.entry.value.content as { index: number; options: string[] });
	let selected = $derived(content.options[content.index]);

	function onReset(newValue: ConfigValue) {
		content = newValue.content as { index: number; options: string[] };
		selected = content.options[content.index];
	}

	function onValueChange(value: string) {
		let index = content.options.indexOf(value);
		setConfigEntry(entryId, {
			type: 'enum',
			content: {
				index,
				options: content.options
			}
		});
	}
</script>

<Select
	triggerClass="grow"
	type="single"
	items={selectItems(content.options)}
	bind:value={selected}
	{onValueChange}
	disabled={locked}
/>
<ResetConfigButton {entryId} {onReset} {locked} />
