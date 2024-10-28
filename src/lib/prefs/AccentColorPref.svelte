<script lang="ts">
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Label from '$lib/components/Label.svelte';
	import { colors, getAccentColor, setAccentColor } from '$lib/theme';
	import { capitalize } from '$lib/util';

	let value = getAccentColor();
</script>

<div class="flex items-center">
	<Label text="Accent color">Changes the zoom level of the mod manager.</Label>

	<Dropdown
		class="flex-grow"
		selected={value}
		items={colors}
		getLabel={capitalize}
		onSelectedChange={(newValue) => {
			value = newValue;
			setAccentColor(value);
		}}
		multiple={false}
	/>
</div>
