<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import * as api from '$lib/api';
	import type { Mod } from '$lib/types';
	import { selectItems } from '$lib/util';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import profiles from '$lib/state/profile.svelte';

	let open = $state(false);
	let mod: Mod | null = $state(null);

	let profileName: string = $state('');

	onMount(() => {
		listen<Mod>('install_mod', (evt) => {
			mod = evt.payload;
			profileName = profiles.active?.name ?? profiles.list[0].name;

			open = true;
		});
	});

	async function install() {
		if (mod === null) return;

		let profileIndex = profiles.list.findIndex((profile) => profile.name === profileName);
		if (profileIndex === -1) return;

		open = false;

		await profiles.setActive(profileIndex);
		await api.profile.install.mod({
			packageUuid: mod.uuid,
			versionUuid: mod.versionUuid
		});
	}
</script>

<ConfirmDialog bind:open title="Install {mod?.name}">
	<p class="text-primary-300">Choose a profile to install the mod to:</p>

	<Select
		triggerClass="w-full"
		items={selectItems(profiles.list.map((profile) => profile.name))}
		avoidCollisions={false}
		type="single"
		bind:value={profileName}
	/>

	{#snippet buttons()}
		<Button icon="mdi:download" onclick={install}>Install</Button>
	{/snippet}
</ConfirmDialog>
