<script lang="ts">
	import Label from '$lib/components/ui/Label.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';

	import { ModLoader, type LaunchMode } from '$lib/types';
	import Info from '$lib/components/ui/Info.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { toHeaderCase } from 'js-convert-case';
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';
	import { platform as osPlatform } from '@tauri-apps/plugin-os';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { pushInfoToast } from '$lib/toast';
	import Button from '$lib/components/ui/Button.svelte';

	type Props = {
		platform: string;
		value: LaunchMode;
		set: (value: LaunchMode) => Promise<void>;
	};

	let { platform, value = $bindable(), set }: Props = $props();

	let instances = $derived(value.content?.instances ?? 1);
	let intervalSecs = $derived(value.content?.intervalSecs ?? 10);

	let items = $derived([
		{
			value: 'launcher',
			label: m.launchModePref_mode_launcher({ platform: toHeaderCase(platform) })
		},
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

	// macOS only: Steam must run Gale's launcher script for BepInEx to inject.
	// The script lives in the active profile's directory (also reachable via
	// File > Open profile folder), so the launch option is built from its path.
	const isMacOS = osPlatform() === 'macos';
	let macLaunchOption = $derived(
		`/bin/sh "${profiles.active?.path ?? m.launchModePref_macos_profilePathPlaceholder()}/run_bepinex.sh" %command%`
	);

	async function copyMacLaunchOption() {
		await writeText(macLaunchOption);
		pushInfoToast({ message: m.launchModePref_macos_copied() });
	}
</script>

<div class="flex items-center">
	<Label>{m.launchModePref_title()}</Label>

	<Info>
		<p>{m.launchModePref_content_1()}</p>
		<p class="my-1.5">
			<b>{m.launchModePref_content_2()}</b>
			{m.launchModePref_content_3()}
		</p>
		<p>
			<b>{m.launchModePref_content_4()}</b>
			{m.launchModePref_content_5()}
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

{#if isMacOS && value.type === 'launcher' && games.active?.modLoader === ModLoader.BepInEx}
	<div class="text-primary-700 dark:text-primary-300 mt-1 mb-2 text-sm">
		<p>{m.launchModePref_macos_note()}</p>
		<div class="mt-1.5 flex items-center gap-2">
			<code
				class="text-primary-700 dark:bg-primary-900 dark:text-primary-300 bg-primary-100 min-w-0 grow truncate rounded-md px-3 py-1 font-mono"
				>{macLaunchOption}</code
			>
			<Button color="primary" icon="mdi:content-copy" onclick={copyMacLaunchOption}>
				{m.launchModePref_macos_copy()}
			</Button>
		</div>
	</div>
{/if}

<div class="flex items-center">
	<Label>{m.launchModePref_instance_title()}</Label>

	<Info>
		{m.launchModePref_instance_content_1()}
		<b>{m.launchModePref_instance_content_2()}</b>
		{m.launchModePref_instance_content_3()}
	</Info>

	<InputField
		disabled={value.type === 'launcher'}
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
		{m.launchModePref_interval_content_1()}<b>{m.launchModePref_interval_content_2()}</b
		>{m.launchModePref_interval_content_3()}
	</Info>

	<InputField
		disabled={value.type === 'launcher' || instances <= 1}
		value={intervalSecs.toString()}
		onchange={(value) => {
			intervalSecs = parseInt(value);
			submit();
		}}
	/>
</div>
