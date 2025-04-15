<script lang="ts">
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Label from '$lib/components/Label.svelte';
	import {
		defaultColors,
		getColor,
		setColor,
		type DefaultColor,
		type ColorCategory
	} from '$lib/theme';
	import { capitalize } from '$lib/util';

	export let category: ColorCategory;

	let value = getColor(category);
	let customColor = value.type === 'custom' ? value.hex : '#6b7280';

	let options = ['custom', ...Object.keys(defaultColors)];
	let selected = value.type === 'custom' ? 'custom' : value.name;

	$: if (value.type === 'custom') {
		changeCustomColor(customColor);
	}

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
</script>

<div class="flex items-center">
	<Label>{capitalize(category)} color</Label>

	<Dropdown
		class="grow"
		{selected}
		items={options}
		getLabel={capitalize}
		onSelectedChange={onDropdownChange}
		multiple={false}
	/>

	{#if value.type === 'custom'}
		<input type="color" bind:value={customColor} class="ml-1 h-full grow" />
	{/if}
</div>
