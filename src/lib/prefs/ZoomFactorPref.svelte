<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import Select from '$lib/components/Select.svelte';

	const options = [0.5, 0.75, 1, 1.25, 1.5];
	const items = options.map((option) => ({
		label: option * 100 + '%',
		value: option.toFixed(2)
	}));

	type Props = {
		value: number;
		set: (newValue: number) => void;
	};

	let { value = $bindable(), set }: Props = $props();
</script>

<div class="flex items-center">
	<Label>Zoom factor</Label>

	<Select
		type="single"
		triggerClass="grow"
		{items}
		value={value.toFixed(2)}
		onValueChange={(newValue) => {
			value = parseFloat(newValue);
			set(value);
		}}
	/>
</div>
