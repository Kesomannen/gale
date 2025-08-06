<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import TabsMenu from '$lib/components/ui/TabsMenu.svelte';

	import { Tabs } from 'bits-ui';

	import type { AnyImportData, ImportData, SyncImportData } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { readText } from '@tauri-apps/plugin-clipboard-manager';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import ModCardList from '$lib/components/ui/ModCardList.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Info from '$lib/components/ui/Info.svelte';
	import { onDestroy, onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { discordAvatarUrl, selectItems } from '$lib/util';
	import { pushInfoToast } from '$lib/toast';
	import Select from '$lib/components/ui/Select.svelte';
	import * as api from '$lib/api';
	import profiles from '$lib/state/profile.svelte';
	import games from '$lib/state/game.svelte';
	import SyncAvatar from '../ui/SyncAvatar.svelte';

	const uuidRegex =
		/^[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}$/i;

	let open: boolean = $state(false);
	let data: AnyImportData | null = $state(null);

	let key: string = $state('');
	let name: string = $state('');
	let loading: boolean = $state(false);
	let importAll: boolean = $state(false);
	let mode: 'new' | 'overwrite' = $state('new');

	let unlistenFn: UnlistenFn | undefined;

	onMount(async () => {
		unlistenFn = await listen<AnyImportData>('import_profile', (evt) => {
			data = evt.payload;
			name = data.manifest.profileName;
			mode = isAvailable(name) ? 'new' : 'overwrite';

			open = true;
		});
	});

	onDestroy(() => {
		unlistenFn?.();
	});

	async function getKeyFromClipboard() {
		key = (await readText()) ?? '';
	}

	async function submitKey() {
		loading = true;

		let type = uuidRegex.test(key.trim()) ? 'normal' : 'sync';

		try {
			if (type === 'normal') {
				data = {
					type: 'normal',
					...(await api.profile.import.readCode(key.trim()))
				};
			} else {
				data = {
					type: 'sync',
					...(await api.profile.sync.read(key.trim()))
				};
			}

			await openFor(data);
		} finally {
			loading = false;
		}
	}

	async function importData() {
		if (!data) return;

		if (mode === 'overwrite') {
			let profile = profiles.list.find((profile) => profile.name === name);

			if (profile?.modCount ?? 0 > 0) {
				let confirmed = await confirm(`Are you sure you want to override ${name}?`);
				if (!confirmed) return;
			}
		}

		open = false;

		if (data.type === 'normal') {
			data.manifest.profileName = name;

			await api.profile.import.profile(data, importAll);
		} else {
			await api.profile.sync.clone(data.id, name);
		}

		data = null;
		importAll = false;

		await pushInfoToast({ message: `Imported profile ${name}.` });
	}

	function isAvailable(name: string) {
		return !profiles.list.some((profile) => profile.name === name);
	}

	export async function openFor(importData: AnyImportData) {
		data = importData;

		if (data.manifest.community !== null && games.active?.slug !== data.manifest.community) {
			await games.setActive(data.manifest.community);
		}

		name = data.manifest.profileName;
		mode = isAvailable(name) ? 'new' : 'overwrite';
		open = true;
	}

	export function openForCode() {
		data = null;
		getKeyFromClipboard();

		open = true;
	}

	let mods = $derived(data?.manifest.mods ?? []);
	let nameAvailable = $derived(mode === 'overwrite' || isAvailable(name));

	$effect(() => {
		if (mode === 'overwrite' && isAvailable(name)) {
			name = profiles.list[0].name;
		}
	});
</script>

<Dialog
	title="Import profile"
	bind:open
	onclose={() => {
		data = null;
		importAll = false;
	}}
>
	{#if data === null}
		<div class="mt-1 flex items-center gap-2">
			<div class="grow">
				<InputField bind:value={key} class="w-full" size="lg" placeholder="Enter import code..." />
			</div>

			<Button onclick={submitKey} {loading} icon="mdi:import">Import</Button>
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
							<Tooltip
								class="absolute right-2 bottom-0 h-full cursor-default text-xl text-red-500"
								text="Profile {name} already exists!"
							>
								<Icon icon="mdi:error" />
							</Tooltip>
						{/if}
					</div>
				</div>
			</Tabs.Content>

			<Tabs.Content value="overwrite">
				<div class="flex items-center">
					<Label>Choose profile</Label>

					<Info>Which existing profile to overwrite with the imported one.</Info>

					<Select
						triggerClass="grow"
						items={selectItems(profiles.list.map((profile) => profile.name))}
						type="single"
						avoidCollisions={false}
						bind:value={name}
					/>
				</div>
			</Tabs.Content>
		</TabsMenu>

		<details>
			<summary class="text-primary-300 mt-2 cursor-pointer">{mods.length} mods to install</summary>

			<ModCardList
				names={mods.map(
					(mod) => `${mod.name}-${mod.version.major}.${mod.version.minor}.${mod.version.patch}`
				)}
				class="mt-2 max-h-[50vh] shrink grow"
			/>
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
				<Checkbox bind:checked={importAll} />
			</div>
		</details>

		{#if data.type === 'sync'}
			<Tooltip
				text="This profile will receive automatic updates from its owner."
				class="cursor-help"
			>
				<div class="text-primary-300 mt-2 flex items-center gap-2">
					<Icon icon="mdi:info" />
					<div>Synced profile</div>
				</div>
			</Tooltip>

			<div class="mt-1 flex items-center gap-2">
				<SyncAvatar user={data.owner} />
				<div class="text-primary-300">
					Owned by {data.owner.displayName}
				</div>
			</div>
		{/if}

		<div class="mt-2 flex w-full items-center justify-end gap-2">
			<Button
				color="primary"
				onclick={() => {
					open = false;
					data = null;
				}}>Cancel</Button
			>
			<Button disabled={!nameAvailable} {loading} onclick={importData} icon="mdi:import"
				>Import</Button
			>
		</div>
	{/if}
</Dialog>
