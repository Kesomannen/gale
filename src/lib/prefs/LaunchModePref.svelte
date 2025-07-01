<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import InputField from '$lib/components/InputField.svelte';

	import type { LaunchMode } from '$lib/types';
	import { selectItems } from '$lib/util';
	import { activeGame } from '$lib/stores.svelte';
	import Info from '$lib/components/Info.svelte';
	import Select from '$lib/components/Select.svelte';
	import { toSentenceCase } from 'js-convert-case';

	type Props = {
		value: LaunchMode;
		set: (value: LaunchMode) => Promise<void>;
	};

	let { value = $bindable(), set }: Props = $props();

	let instances = $state(value.content?.instances ?? 1);
	let intervalSecs = $state(value.content?.intervalSecs ?? 10);

	async function onValueChange(newValue: string) {
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

	<Select
		type="single"
		triggerClass="grow"
		items={selectItems(['launcher', 'direct'], toSentenceCase)}
		value={value?.type ?? 'direct'}
		disabled={platforms.length === 0}
		{onValueChange}
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
		onchange={(value) => {
			instances = parseInt(value);
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
		onchange={(value) => {
			intervalSecs = parseInt(value);
			submit();
		}}
	/>
</div>
