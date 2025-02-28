<script lang="ts">
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Info from '$lib/components/Info.svelte';
	import Label from '$lib/components/Label.svelte';

	export let value: number;
	export let set: (newValue: number) => void;
</script>

<div class="flex items-center">
	<Label>Zoom factor</Label>

	<Dropdown
		class="grow"
		items={[0.5, 0.75, 1, 1.25, 1.5]}
		selected={value}
		onSelectedChange={(newValue) => {
			value = newValue;
			set(newValue);
		}}
		getLabel={(percentage) => percentage * 100 + '%'}
		multiple={false}
	/>
</div>
