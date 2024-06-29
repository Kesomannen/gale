<script lang="ts">
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Label from '$lib/components/Label.svelte';

	export let value: number;
	export let set: (newValue: number) => void;
</script>

<div class="flex items-center">
	<Label text="Zoom factor">Changes the zoom level of the mod manager.</Label>

	<Dropdown
		class="flex-grow"
		items={[0.5, 0.75, 1, 1.25, 1.5]}
		selected={value}
		onSelectedChangeSingle={(newValue) => {
			value = newValue;
			set(newValue);
		}}
		getLabel={(percentage) => percentage * 100 + '%'}
	/>
</div>
