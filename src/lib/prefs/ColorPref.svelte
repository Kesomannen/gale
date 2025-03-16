<script lang="ts">
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Label from '$lib/components/Label.svelte';
	import { colorValues, getColor, setColor, type Color, type ColorCategory } from '$lib/theme';
	import { capitalize } from '$lib/util';

	export let category: ColorCategory;
	export let fallback: Color;

	let value = getColor(category, fallback);

	const colorNames = Object.keys(colorValues) as Color[];
</script>

<div class="flex items-center">
	<Label>{capitalize(category)} color</Label>

	<Dropdown
		class="grow"
		selected={value}
		items={colorNames}
		getLabel={capitalize}
		onSelectedChange={(newValue) => {
			value = newValue;
			setColor(category, value);
		}}
		multiple={false}
	/>
</div>
