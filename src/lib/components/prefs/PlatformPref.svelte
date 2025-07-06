<script lang="ts">
	import Label from '$lib/components/ui/Label.svelte';

	import type { Platform } from '$lib/types';
	import { selectItems } from '$lib/util';
	import Info from '$lib/components/ui/Info.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { toHeaderCase } from 'js-convert-case';
	import games from '$lib/state/game.svelte';

	type Props = {
		value: Platform | null;
		set: (value: Platform) => Promise<void>;
	};

	let { value = $bindable(), set }: Props = $props();

	let platforms = $derived(games.active?.platforms ?? []);
</script>

<div class="flex items-center">
	<Label>Platform</Label>

	<Info>The platform where your game is installed.</Info>

	<Select
		type="single"
		triggerClass="grow"
		items={selectItems(platforms, toHeaderCase)}
		value={value ?? platforms[0]}
		disabled={platforms.length === 1}
		onValueChange={(newValue) => {
			value = newValue as Platform;
			set(value);
		}}
	/>
</div>
