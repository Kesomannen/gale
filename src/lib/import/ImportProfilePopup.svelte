<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';

	import { Tabs } from 'bits-ui';

	import { invokeCommand } from '$lib/invoke';
	import type { ImportData } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { clipboard, dialog } from '@tauri-apps/api';
	import InputField from '$lib/components/InputField.svelte';
	import { profiles, refreshProfiles } from '$lib/stores';
	import BigButton from '$lib/components/BigButton.svelte';
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';

	export let open: boolean;
	export let data: ImportData | null;

	let key: string;
	let name: string;
	let loading: boolean;
	let mode: 'new' | 'overwrite' = 'new';

	$: if (open) {
		getKeyFromClipboard();
	}

	$: if (mode === 'overwrite' && isAvailable(name)) {
		name = profiles[0].name;
	}

	$: nameAvailable = mode === 'overwrite' || isAvailable(name);

	async function getKeyFromClipboard() {
		key = (await clipboard.readText()) ?? '';
	}

	async function submitKey() {
		loading = true;
		try {
			data = await invokeCommand<ImportData>('import_code', { key: key.trim() });
			name = data.name;
			mode = isAvailable(name) ? 'new' : 'overwrite';
		} catch (e) {
			open = false;
		} finally {
			loading = false;
		}
	}

	async function importData() {
		if (!data) return;
		
		data.name = name;

		if (mode === 'overwrite') {
			let confirmed = await dialog.confirm(`Are you sure you want to override '${data.name}'?`, {
				title: 'Overwrite profile'
			});

			if (!confirmed) return;
		}

		invokeCommand('import_data', { data }).then(refreshProfiles);
		data = null;
		open = false;
	}

	function isAvailable(name: string) {
		return !profiles.some((profile) => profile.name === name);
	}
</script>

<Popup title="Import profile" bind:open onClose={() => (data = null)}>
	{#if data}
		<TabsMenu
			bind:value={mode}
			options={[
				{ value: 'new', label: 'Create new' },
				{ value: 'overwrite', label: 'Overwrite existing' }
			]}
		>
			<Tabs.Content value="new">
				<InputField label="Profile name" bind:value={name} />
				{#if !nameAvailable}
					<div class="flex items-center gap-1 text-red-400 text-md font-bold pl-52 mt-1">
						<Icon icon="mdi:error" class="text-lg" />
						Profile '{name}' already exists
					</div>
				{/if}
			</Tabs.Content>

			<Tabs.Content value="overwrite">
				<div class="flex items-center">
					<Label text="Choose profile" />

					<Dropdown
						class="flex-grow"
						items={profiles.map((profile) => profile.name)}
						avoidCollisions={false}
						bind:selected={name}
					/>
				</div>
			</Tabs.Content>
		</TabsMenu>

		<div class="flex w-full justify-end items-center mt-1 gap-2 text-slate-400">
			{data.mods.length} mods will be installed

			<BigButton disabled={!nameAvailable || loading} on:click={importData}>Import</BigButton>
		</div>
	{:else}
		<div class="flex gap-2 mt-1">
			<InputField bind:value={key} size="lg"placeholder="Enter import code..." />

			<BigButton on:click={submitKey} disabled={loading}>
				{#if loading}
					<Icon icon="mdi:loading" class="animate-spin" />
				{:else}
					Import
				{/if}
			</BigButton>
		</div>
	{/if}
</Popup>
