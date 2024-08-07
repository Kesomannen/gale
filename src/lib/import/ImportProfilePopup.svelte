<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';

	import { Tabs } from 'bits-ui';

	import { invokeCommand } from '$lib/invoke';
	import type { ImportData } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { readText } from '@tauri-apps/plugin-clipboard-manager';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import InputField from '$lib/components/InputField.svelte';
	import { profiles, refreshProfiles } from '$lib/stores';
	import BigButton from '$lib/components/BigButton.svelte';
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import ModCardList from '$lib/modlist/ModCardList.svelte';

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
		key = (await readText()) ?? '';
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
			let confirmed = await confirm(`Are you sure you want to override '${data.name}'?`, {
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
				<div class="flex items-center">
					<Label text="Profile name" />

					<InputField bind:value={name} class="w-full" />
				</div>

				{#if !nameAvailable}
					<div class="flex items-center gap-1 text-red-400 text-md font-bold mt-1">
						<div class="w-[30%] min-w-52" />
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

		{#if data.modNames}
			<h3 class="text-white text-lg font-semibold mt-2">{data.mods.length} mods to install</h3>
			<ModCardList names={data.modNames} class="max-h-[55lvh] mt-2" />
		{/if}

		<div class="flex w-full justify-end items-center mt-2 gap-2 text-slate-400">
			<BigButton color="gray" on:click={() => {
				open = false;
				data = null;
			}}>Cancel</BigButton>
			<BigButton disabled={!nameAvailable || loading} on:click={importData}>Import</BigButton>
		</div>
	{:else}
		<div class="flex gap-2 mt-1">
			<div class="flex-grow">
				<InputField bind:value={key} class="w-full" size="lg" placeholder="Enter import code..." />
			</div>

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
