<script lang="ts">
	import Label from '$lib/components/ui/Label.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { m } from '$lib/paraglide/messages';
	import games from '$lib/state/game.svelte';
	import { Backends } from '$lib/types';
	import Info from '../ui/Info.svelte';

	const items = [
		{
			label: 'Thunderstore',
			value: Backends.Thunderstore
		},
		{
			label: 'Thunderstore + Hexium',
			value: Backends.All
		},
		{
			label: 'Hexium',
			value: Backends.Hexium
		}
	];

	type Props = {
		value: Backends;
		set: (newValue: Backends) => Promise<void>;
	};

	let { value = $bindable(), set }: Props = $props();
</script>

<div class="flex items-center">
	<Label>{m.backendPref_title()}</Label>

	<Info>
		{m.backendPref_info()}
	</Info>

	<Select
		type="single"
		triggerClass="grow"
		{items}
		{value}
		onValueChange={async (value) => {
			await set(value as Backends);
			if (games.active) {
				await games.setActive(games.active.slug);
			}
		}}
	/>
</div>
