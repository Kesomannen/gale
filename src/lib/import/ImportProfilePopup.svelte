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
					<Label text="Profile name">A unique name for the imported profile.</Label>

					<div class="relative flex-grow">
						<InputField bind:value={name} class="w-full" />

						{#if !nameAvailable}
							<Tooltip class="absolute bottom-0 right-2 h-full text-xl text-red-500" side="left">
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
					<Label text="Choose profile">
						Which existing profile to overwrite with the imported one.
					</Label>

					<Dropdown
						class="flex-grow"
						items={profiles.map((profile) => profile.name)}
						avoidCollisions={false}
						bind:selected={name}
					/>
				</div>
			</Tabs.Content>
		</TabsMenu>

		{#if data.modNames !== null}
			<details>
				<summary class="mt-2 cursor-pointer text-slate-300"
					>{data.mods.length} mods to install</summary
				>

				<ModCardList names={data.modNames} class="mt-2 max-h-[50vh] flex-shrink flex-grow" />
			</details>
		{/if}

		<div class="mt-2 flex w-full items-center justify-end gap-2 text-slate-400">
			<BigButton
				color="gray"
				on:click={() => {
					open = false;
					data = null;
				}}>Cancel</BigButton
			>
			<BigButton disabled={!nameAvailable || loading} on:click={importData}>Import</BigButton>
		</div>
	{:else}
		<div class="mt-1 flex gap-2">
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
