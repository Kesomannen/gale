<script lang="ts">
	import Label from '$lib/components/ui/Label.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';

	import type { LaunchMode } from '$lib/types';
	import { selectItems } from '$lib/util';
	import Info from '$lib/components/ui/Info.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { toHeaderCase, toSentenceCase } from 'js-convert-case';
	import games from '$lib/state/game.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		platform: string;
		value: LaunchMode;
		set: (value: LaunchMode) => Promise<void>;
	};

	let { platform, value = $bindable(), set }: Props = $props();

	let instances = $state(value.content?.instances ?? 1);
	let intervalSecs = $state(value.content?.intervalSecs ?? 10);

	let items = $derived([
		{ value: 'launcher', label: m.launchModePref_mode_launcher({ platform: toHeaderCase(platform) }) },
		{ value: 'direct', label: m.launchModePref_mode_direct() }
	]);

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

	let platforms = $derived(games.active?.platforms ?? []);
</script>

<div class="flex items-center">
	<Label>{m.launchModePref_title()}</Label>

	<Info>
		<p>{m.launchModePref_content_1()}</p>
		<p class="my-1.5">
			<b>{m.launchModePref_content_2()}</b> {m.launchModePref_content_3()}
		</p>
		<p>
			<b>{m.launchModePref_content_4()}</b> {m.launchModePref_content_5()}
		</p>
	</Info>

	<Select
		type="single"
		triggerClass="grow"
		{items}
		value={value?.type ?? 'direct'}
		disabled={platforms.length === 0}
		{onValueChange}
	/>
</div>

<div class="flex items-center">
	<Label>{m.launchModePref_instance_title()}</Label>

	<Info>
		{m.launchModePref_instance_content_1()}
		<b>{m.launchModePref_instance_content_2()}</b>
		{m.launchModePref_instance_content_3()}
	</Info>

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
	<Label>{m.launchModePref_interval_title()}</Label>

	<Info>
		{m.launchModePref_interval_content_1()}<b>{m.launchModePref_interval_content_2()}</b>{m.launchModePref_interval_content_3()}
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
