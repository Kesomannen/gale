<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import InputField from '$lib/components/InputField.svelte';

	import type { LaunchMode } from '$lib/models';
	import { sentenceCase } from '$lib/util';
	import { activeGame } from '$lib/stores';
	import Info from '$lib/components/Info.svelte';

	type Props = {
		value: LaunchMode;
		set: (value: LaunchMode) => Promise<void>;
	};

	let { value = $bindable(), set }: Props = $props();

	let instances = $state(value.content?.instances ?? 1);
	let intervalSecs = $state(value.content?.intervalSecs ?? 10);

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

	let platforms = $derived($activeGame?.platforms ?? []);
</script>

<div class="flex items-center">
	<Label>Launch mode</Label>

	<Info>
		<p>Determines how the game is launched.</p>
		<p class="my-1.5">
			<b>Launcher:</b> Launches via the specified platform. This is required for some games that, for
			example, require Steam to be running.
		</p>
		<p>
			<b>Direct:</b> Launches the game directly from the executable. Allows you to launch multiple instances
			at once.
		</p>
	</Info>

	<Dropdown
		class="grow"
		items={['launcher', 'direct']}
		getLabel={sentenceCase}
		selected={value?.type ?? 'direct'}
		multiple={false}
		disabled={platforms.length === 0}
		{onSelectedChange}
	/>
</div>

<div class="flex items-center">
	<Label>Number of instances</Label>

	<Info
		>How many instances of the game to launch at once. Only available in <b>Direct</b> mode.</Info
	>

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
	<Label>Interval between launches</Label>

	<Info>
		How many seconds to wait between launching each instance. Only applicable in <b>Direct</b> mode with
		multiple instances.
	</Info>

	<InputField
		disabled={value.type !== 'direct' || instances <= 1}
		value={intervalSecs.toString()}
		on:change={({ detail }) => {
			intervalSecs = parseInt(detail);
			submit();
		}}
	/>
</div>
