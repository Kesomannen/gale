<script lang="ts">
	import Label from '$lib/components/ui/Label.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { m } from '$lib/paraglide/messages';
	import { Backends } from '$lib/types';

	const items = [
		{
			label: "Thunderstore",
			value: Backends.Thunderstore
		},
		{
			label: "Thunderstore + Hexium",
			value: Backends.All
		},
		{
			label: "Hexium",
			value: Backends.Hexium,
		}
	];

	type Props = {
		value: Backends;
		set: (newValue: Backends) => void;
	};

	let { value = $bindable(), set }: Props = $props();
</script>

<div class="flex items-center">
	<Label>{m.backendPref_title()}</Label>

	<Select
		type="single"
		triggerClass="grow"
		{items}
		{value}
		onValueChange={(value) => set(value as Backends)}
	/>
</div>
