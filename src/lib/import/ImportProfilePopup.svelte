<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';

	import { Tabs } from 'bits-ui';

	import { invokeCommand } from '$lib/invoke';
	import type { AnyImportData, ImportData, SyncImportData as SyncImportData } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { readText } from '@tauri-apps/plugin-clipboard-manager';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import InputField from '$lib/components/InputField.svelte';
	import { activeGame, profiles, refreshProfiles, setActiveGame } from '$lib/stores';
	import BigButton from '$lib/components/BigButton.svelte';
	import Label from '$lib/components/Label.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import ModCardList from '$lib/modlist/ModCardList.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import Info from '$lib/components/Info.svelte';
	import { onDestroy, onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { discordAvatarUrl } from '$lib/util';
	import { pushInfoToast } from '$lib/toast';

	const uuidRegex =
		/^[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}$/i;

	let open: boolean;
	let data: AnyImportData | null;

	let key: string;
	let name: string;
	let loading: boolean;
	let importAll: boolean;
	let mode: 'new' | 'overwrite' = 'new';

	let unlistenFn: UnlistenFn | undefined;

	$: mods = data?.manifest.mods ?? [];

	$: if (mode === 'overwrite' && isAvailable(name)) {
		name = profiles[0].name;
	}

	$: nameAvailable = mode === 'overwrite' || isAvailable(name);

	onMount(async () => {
		unlistenFn = await listen<ImportData>('import_profile', (evt) => {
			data = { type: 'normal', ...evt.payload };
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
					...(await invokeCommand<ImportData>('read_profile_code', { key: key.trim() }))
				};
			} else {
				data = {
					type: 'sync',
					...(await invokeCommand<SyncImportData>('read_sync_profile', { id: key.trim() }))
				};
			}

			console.log(data);

			await openFor(data);
		} finally {
			loading = false;
		}
	}

	async function importData() {
		if (!data) return;

		if (mode === 'overwrite') {
			let confirmed = await confirm(`Are you sure you want to override ${name}?`);
			if (!confirmed) return;
		}

		open = false;

		if (data.type === 'normal') {
			data.manifest.profileName = name;

			await invokeCommand('import_profile', { data, importAll });
		} else {
			await invokeCommand('clone_sync_profile', { name, id: data.id });
		}

		data = null;
		importAll = false;

		await refreshProfiles();
		pushInfoToast({ message: `Imported profile ${name}.` });
	}

	function isAvailable(name: string) {
		return !profiles.some((profile) => profile.name === name);
	}

	export async function openFor(importData: AnyImportData) {
		data = importData;

		if (data.manifest.community !== null && $activeGame?.slug !== data.manifest.community) {
			await setActiveGame(data.manifest.community);
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
				<Checkbox bind:value={importAll} />
			</div>
		</details>

		{#if data.type === 'sync'}
			<div class="mt-2 flex items-center">
				<img
					src={discordAvatarUrl(data.owner)}
					alt=""
					class="mr-2 size-10 rounded-full shadow-lg"
				/>
				<div class="text-primary-300">
					Owned by {data.owner.displayName}
				</div>
			</div>
		{/if}

		<div class="mt-2 flex w-full items-center justify-end gap-2">
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
