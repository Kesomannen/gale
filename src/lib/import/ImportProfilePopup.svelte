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
	import Tooltip from '$lib/components/Tooltip.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import Info from '$lib/components/Info.svelte';
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';

	let open: boolean;
	let data: ImportData | null;

	let key: string;
	let name: string;
	let loading: boolean;
	let importAll: boolean;
	let mode: 'new' | 'overwrite' = 'new';

	$: if (mode === 'overwrite' && isAvailable(name)) {
		name = profiles[0].name;
	}

	$: nameAvailable = mode === 'overwrite' || isAvailable(name);

	onMount(() => {
		listen<ImportData>('import_profile', (evt) => {
			data = evt.payload;
			name = data.name;
			mode = isAvailable(name) ? 'new' : 'overwrite';

			open = true;
		});
	});

	async function getKeyFromClipboard() {
		key = (await readText()) ?? '';
	}

	async function submitKey() {
		loading = true;
		try {
			data = await invokeCommand<ImportData>('import_code', { key: key.trim() });
			openFor(data);
		} finally {
			loading = false;
		}
	}

	async function importData() {
		if (!data) return;

		data.name = name;

		if (mode === 'overwrite') {
			let confirmed = await confirm(`Are you sure you want to override ${data.name}?`);

			if (!confirmed) return;
		}

		invokeCommand('import_data', { data, importAll }).then(refreshProfiles);
		data = null;
		importAll = false;
		open = false;
	}

	function isAvailable(name: string) {
		return !profiles.some((profile) => profile.name === name);
	}

	export function openFor(importData: ImportData) {
		data = importData;
		name = data.name;
		mode = isAvailable(name) ? 'new' : 'overwrite';

		open = true;
	}

	export function openForCode() {
		data = null;
		getKeyFromClipboard();

		open = true;
	}
</script>

<Popup
	title="Import profile"
	bind:open
	onClose={() => {
		data = null;
		importAll = false;
	}}
>
	{#if data === null}
		<div class="mt-1 flex gap-2">
			<div class="grow">
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
	{:else}
		<TabsMenu
			bind:value={mode}
			options={[
				{ value: 'new', label: 'Create new' },
				{ value: 'overwrite', label: 'Overwrite existing' }
			]}
		>
			<Tabs.Content value="new">
				<div class="flex items-center">
					<Label>Profile name</Label>

					<Info>A unique name for the imported profile.</Info>

					<div class="relative grow">
						<InputField bind:value={name} class="w-full" />

						{#if !nameAvailable}
							<Tooltip class="absolute right-2 bottom-0 h-full cursor-text text-xl text-red-500">
								<Icon icon="mdi:error" />

								<div slot="tooltip">
									Profile {name} already exists!
								</div>
							</Tooltip>
						{/if}
					</div>
				</div>
			</Tabs.Content>

			<Tabs.Content value="overwrite">
				<div class="flex items-center">
					<Label>Choose profile</Label>

					<Info>Which existing profile to overwrite with the imported one.</Info>

					<Dropdown
						class="grow"
						items={profiles.map((profile) => profile.name)}
						avoidCollisions={false}
						multiple={false}
						bind:selected={name}
					/>
				</div>
			</Tabs.Content>
		</TabsMenu>

		<details>
			<summary class="text-primary-300 mt-2 cursor-pointer"
				>{data.modNames.length} mods to install</summary
			>

			<ModCardList names={data.modNames} class="mt-2 max-h-[50vh] shrink grow" />
		</details>

		<details>
			<summary class="text-primary-300 mt-1 cursor-pointer">Advanced options</summary>

			<div class="mt-1 flex items-center">
				<Label>Import all files</Label>
				<Info>
					Import all files found in the profile, instead of just well-known config file formats.
					This is unsafe and can let an attacker install malware on your system. <b
						>Only enable this for trusted profiles!</b
					>
				</Info>
				<Checkbox bind:value={importAll} />
			</div>
		</details>

		<div class="text-primary-400 mt-2 flex w-full items-center justify-end gap-2">
			<BigButton
				color="primary"
				on:click={() => {
					open = false;
					data = null;
				}}>Cancel</BigButton
			>
			<BigButton disabled={!nameAvailable || loading} on:click={importData}>Import</BigButton>
		</div>
	{/if}
</Popup>
