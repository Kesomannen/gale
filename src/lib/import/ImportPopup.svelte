<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { ImportData } from '$lib/models';
	import { Dialog, Select, Tabs } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import { clipboard, dialog } from '@tauri-apps/api';
	import InputField from '$lib/components/InputField.svelte';
	import { profiles, refreshProfiles } from '$lib/profile';
	import BigButton from '$lib/components/BigButton.svelte';
	import Label from '$lib/components/Label.svelte';

	export let open: boolean;
	export let data: ImportData | undefined;
	
	let key: string;
	let name: string;
	let loading: boolean;
	let mode: 'new' | 'overwrite' = 'new';
	
	let profileDropdownOpen: boolean;
	
	$: if (open) {
		getKeyFromClipboard();
	}

	$: nameAvailable = mode === 'overwrite' || !profiles.includes(name);

	async function getKeyFromClipboard() {
		key = (await clipboard.readText()) ?? '';
	}

	async function submitKey() {
		loading = true;
		try {
			data = await invokeCommand<ImportData>('import_code', { key });
			name = data.name;
			mode = profiles.includes(name) ? 'overwrite' : 'new';
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
		data = undefined;
		open = false;
	}
</script>

<Popup title="Import profile" bind:open onClose={() => (data = undefined)}>
	{#if data}
		<Tabs.Root bind:value={mode}>
			<Tabs.List class="flex p-1 my-1 gap-1 rounded-xl text-slate-300 bg-gray-900">
				<Tabs.Trigger
					value="new"
					class="flex-grow rounded-lg px-2 py-0.5
								hover:bg-gray-800 hover:text-slate-100
								data-[state=active]:bg-gray-700 data-[state=active]:text-slate-100 data-[state=active]:font-semibold"
				>
					Create new
				</Tabs.Trigger>
				<Tabs.Trigger
					value="overwrite"
					class="flex-grow rounded-lg px-2 py-0.5
							hover:bg-gray-800 hover:text-slate-100
							data-[state=active]:bg-gray-700 data-[state=active]:text-slate-100 data-[state=active]:font-semibold"
				>
					Overwrite existing
				</Tabs.Trigger>
			</Tabs.List>
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

					<Select.Root
						items={profiles.map((name) => ({ value: name, label: name }))}
						selected={{ value: name, label: name }}
						onSelectedChange={(selection) => (name = selection?.value ?? name)}
						bind:open={profileDropdownOpen}
					>
						<Select.Trigger
							class="flex items-center flex-grow bg-gray-900 rounded-lg px-3 py-1
											border border-gray-500 border-opacity-0 hover:border-opacity-100"
						>
							<Select.Value class="text-slate-300 text-left w-full" />
							<Icon
								class="text-slate-400 text-xl ml-auto transition-all
																transform origin-center {profileDropdownOpen ? 'rotate-180' : 'rotate-0'}"
								icon="mdi:chevron-down"
							/>
						</Select.Trigger>
						<Select.Content
							class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 rounded-lg border border-gray-600"
							transitionConfig={{ duration: 100 }}
						>
							{#each profiles as profileName}
								<Select.Item
									value={profileName}
									class="flex items-center px-3 py-1 truncate text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
								>
									{profileName}
									{#if profileName === name}
										<Select.ItemIndicator class="ml-auto">
											<Icon icon="mdi:check" class="text-green-400 text-lg" />
										</Select.ItemIndicator>
									{/if}
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>
			</Tabs.Content>
		</Tabs.Root>

		<div class="flex w-full justify-end items-center mt-1 gap-2 text-slate-400">
			{data.mods.length} mods will be installed

			<BigButton disabled={!nameAvailable || loading} onClick={importData}>Import</BigButton>
		</div>
	{:else}
		<div class="flex gap-2 mt-1">
			<InputField bind:value={key} size="lg" placeholder="Enter import code..." />

			<BigButton onClick={submitKey} disabled={loading}>
				{#if loading}
					<Icon icon="mdi:loading" class="animate-spin" />
				{:else}
					Import
				{/if}
			</BigButton>
		</div>
	{/if}
</Popup>
