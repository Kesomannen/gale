<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import InputField from '$lib/components/InputField.svelte';

	import type { LaunchMode } from '$lib/models';
	import { sentenceCase } from '$lib/util';

	export let value: LaunchMode;
	export let set: (value: LaunchMode) => Promise<void>;

	let instances = value.content?.instances ?? 1;
	let intervalSecs = value.content?.intervalSecs ?? 10;

	async function onSelectedChange(newValue: string) {
		value.type = newValue as 'launcher' | 'direct';
		await submit();
	}

	async function submit() {
		if (value.type === 'direct') {
			value.content = { instances, intervalSecs };
		} else {
			value.content = undefined;
		}

		await set(value);
	}
</script>

<div class="flex items-center">
	<Label text="Launch mode">
		<p>Determines how the game is launched.</p>
		<p class="my-1.5">
			<b>Launcher:</b> Launches through the specified platform.
		</p>
		<p>
			<b>Direct:</b> Launches the game directly from the executable. Allows you to launch multiple instances
			at once.
		</p>
	</Label>

	<Dropdown
		class="flex-grow"
		items={['launcher', 'direct']}
		getLabel={sentenceCase}
		selected={value?.type ?? 'steam'}
		{onSelectedChange}
		multiple={false}
	/>
</div>

<div class="flex items-center">
	<Label text="Number of instances">
		How many instances of the game to launch at once. Only available in direct mode.
	</Label>

	<InputField
		disabled={value.type !== 'direct'}
		value={instances.toString()}
		on:change={({ detail }) => {
			instances = parseInt(detail);
			submit();
		}}
	/>
</div>

<div class="flex items-center">
	<Label text="Interval between launches">
		How many seconds to wait between launching each instance. Only applicable in direct mode with
		multiple instances.
	</Label>

	<InputField
		disabled={value.type !== 'direct' || instances <= 1}
		value={intervalSecs.toString()}
		on:change={({ detail }) => {
			intervalSecs = parseInt(detail);
			submit();
		}}
	/>
</div>
