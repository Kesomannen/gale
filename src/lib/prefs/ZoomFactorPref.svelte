<script lang="ts">
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Label from '$lib/components/Label.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { PrefValue } from '$lib/models';
	import { onMount } from 'svelte';

	let value: number;

	onMount(async () => {
		value = (await invokeCommand<PrefValue>('get_pref', { key: 'zoom_factor' })) as number;
	});

	function set(newValue: number) {
		value = newValue;
		invokeCommand('set_pref', { key: 'zoom_factor', value });
	}
</script>

<div class="flex items-center">
	<Label text="Zoom factor">Changes the zoom level of the mod manager.</Label>

	<Dropdown
		class="flex-grow"
		items={[0.5, 0.75, 1, 1.25, 1.5]}
		selected={value}
		onSelectedChangeSingle={set}
		getLabel={(percentage) => percentage * 100 + '%'}
	/>
</div>
