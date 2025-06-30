<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';

	import type { Platform } from '$lib/models';
	import { titleCase } from '$lib/util';
	import { activeGame } from '$lib/stores';
	import Info from '$lib/components/Info.svelte';

	type Props = {
		value: Platform | null;
		set: (value: Platform) => Promise<void>;
	};

	let { value = $bindable(), set }: Props = $props();

	let platforms = $derived($activeGame?.platforms ?? []);
</script>

<div class="flex items-center">
	<Label>Platform</Label>

	<Info>The platform where your game is installed.</Info>

	<Dropdown
		class="grow"
		items={platforms}
		getLabel={titleCase}
		selected={value ?? platforms[0]}
		disabled={platforms.length === 1}
		multiple={false}
		onSelectedChange={(newValue) => {
			value = newValue;
			set(value);
		}}
	/>
</div>
