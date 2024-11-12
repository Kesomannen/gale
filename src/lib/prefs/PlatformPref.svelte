<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';

	import type { Platform } from '$lib/models';
	import { sentenceCase, titleCase } from '$lib/util';
	import { activeGame } from '$lib/stores';

	export let value: Platform | null;
	export let set: (value: Platform) => Promise<void>;

	$: platforms = $activeGame?.platforms ?? [];
</script>

<div class="flex items-center">
	<Label text="Platform">The platform where your game is hosted.</Label>

	<Dropdown
		class="flex-grow"
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
