<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { LaunchMode, PrefValue } from '$lib/models';
	import { onMount } from 'svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';

	let value: LaunchMode | undefined;
	let instances = 1;

	onMount(async () => {
		value = (await invokeCommand<PrefValue>('get_pref', { key: 'launch_mode' })) as LaunchMode;
		instances = value.content?.instances ?? 1;
	});

	function set(newValue: 'steam' | 'direct') {
		if (!value) return;
		value.type = newValue;

		if (value.type == 'direct') {
			value.content = { instances };
		} else {
			value.content = undefined;
		}

		invokeCommand('set_pref', { key: 'launch_mode', value });
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
		selected={value?.type ?? 'steam'}
		onSelectedChangeSingle={set}
	/>

	<Tooltip text="Number of instances to launch. Only available in direct mode." side="top">
		<input
			type="number"
			step="int32"
			disabled={value?.type !== 'direct'}
			bind:value={instances}
			on:input={() => set('direct')}
			class="px-3 py-1 rounded-lg bg-gray-900 ml-1
					text-slate-300 hover:text-slate-100 disabled:text-slate-400 border border-gray-500 border-opacity-0 
					enabled:hover:border-opacity-100"
		/>
	</Tooltip>
</div>
