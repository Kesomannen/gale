<script lang="ts">
	import Select from '$lib/components/ui/Select.svelte';
	import { setConfigEntry } from '$lib/config';
	import { m } from '$lib/paraglide/messages';
	import type { ConfigEntryId, ConfigValue } from '$lib/types';
	import { selectItems } from '$lib/util';
	import ResetConfigButton from './ResetConfigButton.svelte';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
	};

	let { entryId, locked }: Props = $props();

	let value = entryId.entry.value;
	let content = $state(value.content as { indicies: number[]; options: string[] });
	let selected = $derived(content.indicies.map((index) => content.options[index]));

	function onReset(newValue: ConfigValue) {
		content = newValue.content as { indicies: number[]; options: string[] };
		selected = content.indicies.map((index) => content.options[index]);
	}

	function onValueChange(newValues: string[]) {
		content.indicies = newValues.map((value) => content.options.indexOf(value));
		setConfigEntry(entryId, {
			type: 'flags',
			content
		});
	}
</script>

<Select
	type="multiple"
	triggerClass="grow overflow-hidden"
	placeholder={m.flagsConfig_placeholder()}
	items={selectItems(content.options)}
	disabled={locked}
	{onValueChange}
	bind:value={selected}
/>
<ResetConfigButton {entryId} {onReset} {locked} />
