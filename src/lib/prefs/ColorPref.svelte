<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import {
		defaultColors,
		getColor,
		setColor,
		type DefaultColor,
		type ColorCategory
	} from '$lib/theme';
	import { capitalize, selectItems } from '$lib/util';
	import Select from '$lib/components/Select.svelte';

	type Props = {
		category: ColorCategory;
	};

	let { category }: Props = $props();

	let value = $state(getColor(category));
	let customColor = $state(value.type === 'custom' ? value.hex : '#6b7280');

	let options = ['custom', ...Object.keys(defaultColors)];
	let selected = value.type === 'custom' ? 'custom' : value.name;

	function onDropdownChange(newValue: string) {
		if (newValue === 'custom') {
			value = { type: 'custom', hex: customColor };
		} else {
			value = { type: 'default', name: newValue as DefaultColor };
			setColor(category, value);
		}
	}

	function changeCustomColor(hex: string) {
		setColor(category, { type: 'custom', hex });
	}

	$effect(() => {
		if (value.type === 'custom') {
			changeCustomColor(customColor);
		}
	});
</script>

<div class="flex items-center">
	<Label>{capitalize(category)} color</Label>

	<Select
		type="single"
		triggerClass="grow"
		value={selected}
		items={selectItems(options, capitalize)}
		onValueChange={onDropdownChange}
	/>

	{#if value.type === 'custom'}
		<input type="color" bind:value={customColor} class="ml-1 h-full grow" />
	{/if}
</div>
