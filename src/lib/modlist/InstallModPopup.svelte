<script lang="ts">
	import BigButton from '$lib/components/Button.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { Mod } from '$lib/models';
	import { activeProfile, profiles, setActiveProfile } from '$lib/stores';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	let open = $state(false);
	let mod: Mod | null = $state(null);

	let profileName: string = $state();

	onMount(() => {
		listen<Mod>('install_mod', (evt) => {
			mod = evt.payload;
			profileName = $activeProfile?.name ?? profiles[0].name;

			open = true;
		});
	});

	async function install() {
		if (mod === null) return;

		let profileIndex = profiles.findIndex((profile) => profile.name === profileName);
		if (profileIndex === -1) return;

		open = false;

		await setActiveProfile(profileIndex);
		await invokeCommand('install_mod', {
			modRef: {
				packageUuid: mod.uuid,
				versionUuid: mod.versionUuid
			}
		});
	}
</script>

<ConfirmPopup bind:open title="Install {mod?.name}">
	<p class="text-primary-300">Choose a profile to install the mod to:</p>

	<Dropdown
		class="w-full"
		items={profiles.map((profile) => profile.name)}
		avoidCollisions={false}
		multiple={false}
		bind:selected={profileName}
	/>

	{#snippet buttons()}
		<BigButton on:click={install}>Install</BigButton>
	{/snippet}
</ConfirmPopup>
