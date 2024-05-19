<script lang="ts">
	import Popup from '$lib/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { ImportData } from '$lib/models';
	import { Button, Dialog, Select, Tabs } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import { clipboard } from '@tauri-apps/api';
	import InputField from '$lib/InputField.svelte';
	import { profileNames } from '$lib/profile';
	import { slide } from 'svelte/transition';

	export let open: boolean;
	export let data: ImportData | undefined;

	let key: string;
	let loading: boolean;

	let name: string;
	$: if (data && !name) {
		name = data.name;
	}

	let mode: 'new' | 'overwrite' = 'new';

	$: if (open) {
		getKeyFromClipboard();
	}

	async function getKeyFromClipboard() {
		key = (await clipboard.readText()) ?? '';
	}

	async function submitKey() {
		loading = true;
		try {
			data = await invokeCommand<ImportData>('import_code', { key });
		} catch (e) {
			open = false;
		} finally {
			loading = false;
		}
	}
</script>

<Popup title="Import profile" bind:open>
	{#if data}
		<Tabs.Root bind:value={mode}>
			<Tabs.List class="flex p-1 my-1 gap-1 rounded-xl text-slate-300 bg-gray-900">
				<Tabs.Trigger
					value="new"
					class="flex-grow rounded-lg px-2 py-0.5
				                               hover:bg-gray-800 hover:text-slate-100
																			 data-[state=active]:bg-gray-700 data-[state=active]:text-slate-100 data-[state=active]:font-semibold"
				>
					New profile
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
			</Tabs.Content>
			<Tabs.Content value="overwrite">
				<div class="flex items-center">
					<div class="text-slate-300 w-36 truncate">Choose profile</div>

					<Select.Root
						items={profileNames.map((name) => ({ value: name, label: name }))}
						selected={{ value: name, label: name }}
					>
						<Select.Trigger
							class="flex items-center flex-grow bg-gray-900 rounded-lg px-3 py-1
											border border-gray-500 border-opacity-0 hover:border-opacity-100"
						>
							<Select.Value class="text-slate-300 text-left w-full" />
							<Icon
								class="text-slate-400 text-xl ml-auto transition-all
																transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
								icon="mdi:chevron-down"
							/>
						</Select.Trigger>
						<Select.Content
							class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 rounded-lg border border-gray-600"
							transition={slide}
							transitionConfig={{ duration: 100 }}
						>
							{#each profileNames as profileName}
								<Select.Item
									value="direct"
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
			
			<Dialog.Close>
				<Button.Root
					class="rounded-xl px-4 py-2 font-medium text-white bg-green-600 hover:bg-green-500"
				>
					Import
				</Button.Root>
			</Dialog.Close>
		</div>
	{:else}
		<div class="flex mt-2">
			<input
				type="text"
				bind:value={key}
				on:keydown={(e) => {
					if (e.key !== 'Enter') return;
					submitKey();
				}}
				placeholder="Enter import code..."
				class="w-full px-3 py-2 rounded-lg bg-gray-900 text-slate-100 select-none"
			/>
			<Button.Root
				class="rounded-xl px-4 py-2 ml-3 text-slate-100 bg-green-600 hover:bg-green-500
							disabled:bg-gray-600/80 disabled:hover:bg-gray-600/80 disabled:text-gray-200/80"
				on:click={submitKey}
				disabled={loading}
			>
				{#if loading}
					<Icon icon="mdi:loading" class="animate-spin" />
				{:else}
					Import
				{/if}
			</Button.Root>
		</div>
	{/if}
</Popup>
