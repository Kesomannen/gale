<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import type { LaunchMode } from '$lib/models';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { sentenceCase } from '$lib/util';

	export let value: LaunchMode;
	export let set: (value: LaunchMode) => void;

	let instances = value.content?.instances ?? 1;

	function onSelectedChangeSingle(newValue: string) {
		let type = newValue as 'steam' | 'direct';

		if (type == 'steam') {
			value = { type };
		} else {
			value = { type, content: { instances } };
		}

		set(value);
	}
</script>

<div class="flex items-center">
	<Label text="Launch mode">
		<p>Determines how the game is launched.</p>
		<p class="my-1.5">
			<b>Steam:</b> Launches through Steam, which slower than directly. However, some games require Steam
			to be running, including Lethal Company.
		</p>
		<p>
			<b>Direct:</b> Launches the game directly from the executable. Also allows you to launch multiple
			instances at once.
		</p>
	</Label>

	<Dropdown
		class="flex-grow"
		items={['steam', 'direct']}
		getLabel={sentenceCase}
		selected={value?.type ?? 'steam'}
		{onSelectedChangeSingle}
	/>

	<Tooltip text="Number of instances to launch. Only available in direct mode." side="top">
		<input
			type="number"
			step="int32"
			disabled={value?.type !== 'direct'}
			bind:value={instances}
			on:input={() => {
				value.content = { instances };
				set(value);
			}}
			class="px-3 py-1 rounded-lg bg-gray-900 ml-1
					text-slate-300 hover:text-slate-100 disabled:text-slate-400 border border-gray-500 border-opacity-0 
					enabled:hover:border-opacity-100"
		/>
	</Tooltip>
</div>
