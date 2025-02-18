<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import DependantsPopup from '$lib/menu/DependantsPopup.svelte';
	import {
		type Mod,
		type ModActionResponse,
		type ProfileQuery,
		type AvailableUpdate,
		SortBy,
		type Dependant,
		SortOrder,
		type ModContextItem
	} from '$lib/models';
	import ModList from '$lib/modlist/ModList.svelte';
	import { activeProfile, profileQuery, refreshProfiles } from '$lib/stores';
	import { isOutdated } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import Popup from '$lib/components/Popup.svelte';
	import ModCardList from '$lib/modlist/ModCardList.svelte';
	import ProfileModListItem from '$lib/modlist/ProfileModListItem.svelte';
	import UpdateAllBanner from '$lib/modlist/UpdateAllBanner.svelte';
	import { emit } from '@tauri-apps/api/event';
	import Link from '$lib/components/Link.svelte';

	const sortOptions = [
		SortBy.Custom,
		SortBy.InstallDate,
		SortBy.LastUpdated,
		SortBy.Newest,
		SortBy.DiskSpace,
		SortBy.Name,
		SortBy.Author,
		SortBy.Rating,
		SortBy.Downloads
	];

	const contextItems: ModContextItem[] = [
		{
			label: 'Uninstall',
			icon: 'mdi:delete',
			onclick: (mod) =>
				uninstall({
					uuid: mod.uuid,
					fullName: mod.name
				})
		},
		{
			label: 'Change version',
			icon: 'mdi:edit',
			onclick: () => {},
			showFor: (mod) => mod.versions.length > 1,
			children: (mod) =>
				mod.versions.map((version) => ({
					label: version.name,
					onclick: () => updateMod(mod, version.uuid)
				}))
		},
		{
			label: 'Show dependants',
			icon: 'mdi:source-branch',
			onclick: openDependants
		},
		{
			label: 'Open folder',
			icon: 'mdi:folder',
			onclick: (mod) => invokeCommand('open_mod_dir', { uuid: mod.uuid })
		}
	];

	let mods: Mod[] = [];
	let totalModCount = 0;
	let unknownMods: Dependant[] = [];
	let updates: AvailableUpdate[] = [];

	let modList: ModList;
	let maxCount: number;
	let selectedMod: Mod | null = null;

	let removeDependants: DependantsPopup;
	let disableDependants: DependantsPopup;
	let enableDependencies: DependantsPopup;

	let dependantsOpen = false;
	let dependants: string[];

	let activeMod: Mod | null = null;

	$: if (maxCount > 0) {
		$activeProfile;
		$profileQuery;
		refresh();
	}

	$: reorderable =
		$profileQuery.sortBy === SortBy.Custom &&
		$profileQuery.searchTerm === '' &&
		$profileQuery.excludeCategories.length === 0 &&
		$profileQuery.includeCategories.length === 0 &&
		$profileQuery.includeDeprecated &&
		$profileQuery.includeNsfw &&
		$profileQuery.includeDisabled;

	let hasRefreshed = false;
	let refreshing = false;

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		let result = await invokeCommand<ProfileQuery>('query_profile', {
			args: { ...$profileQuery, maxCount }
		});

		mods = result.mods;
		totalModCount = result.totalModCount;
		unknownMods = result.unknownMods;
		updates = result.updates;

		refreshing = false;
		hasRefreshed = true;
	}

	async function toggleMod(mod: Mod, newState: boolean) {
		let response = await invokeCommand<ModActionResponse>('toggle_mod', {
			uuid: mod.uuid
		});

		if (response.type == 'done') {
			refresh();
			return;
		}

		if (newState) {
			enableDependencies.openFor(mod, response.dependants);
		} else {
			disableDependants.openFor(mod, response.dependants);
		}
	}

	async function uninstall(mod: Dependant) {
		let response = await invokeCommand<ModActionResponse>('remove_mod', { uuid: mod.uuid });

		if (response.type == 'done') {
			selectedMod = null;
			await refreshProfiles();
		} else {
			removeDependants.openFor(mod, response.dependants);
		}
	}

	async function openDependants(mod: Mod) {
		dependants = await invokeCommand<string[]>('get_dependants', {
			uuid: mod.uuid
		});

		activeMod = mod;
		dependantsOpen = true;
	}

	async function updateMod(mod: Mod | null, versionUuid?: string) {
		if (mod === null) return;

		if (versionUuid === undefined) {
			await invokeCommand('update_mods', { uuids: [mod.uuid], respectIgnored: false });
		} else {
			await invokeCommand('change_mod_version', {
				modRef: {
					packageUuid: mod.uuid,
					versionUuid: versionUuid
				}
			});
		}

		await refresh();

		selectedMod = mods.find((mod) => mod.uuid === selectedMod!.uuid) ?? null;
		console.log(selectedMod);
	}

	let reorderUuid: string;
	let reorderPrevIndex: number;

	function onDragStart(evt: DragEvent) {
		if (!reorderable || evt.dataTransfer === null) return;

		let element = evt.currentTarget as HTMLElement;

		reorderUuid = element.dataset.uuid!;
		reorderPrevIndex = parseInt(element.dataset.index!);

		evt.dataTransfer.effectAllowed = 'move';
		evt.dataTransfer.setData('text/html', element.outerHTML);
	}

	async function onDragOver(evt: DragEvent) {
		if (!reorderable) return;

		let target = evt.currentTarget as HTMLElement;
		let newIndex = parseInt(target.dataset.index!);
		let delta = newIndex - reorderPrevIndex;

		if (delta === 0) {
			return;
		}

		let temp = mods[reorderPrevIndex];
		mods[reorderPrevIndex] = mods[newIndex];
		mods[newIndex] = temp;

		reorderPrevIndex = newIndex;

		if ($profileQuery.sortOrder === SortOrder.Descending) {
			delta *= -1; // list is reversed
		}

		await emit('reorder_mod', { uuid: reorderUuid, delta });
	}

	async function onDragEnd(evt: DragEvent) {
		if (!reorderable) return;

		await emit('finish_reorder');
	}
</script>

<ModList
	{sortOptions}
	{contextItems}
	queryArgs={profileQuery}
	bind:this={modList}
	bind:mods
	bind:maxCount
	bind:selected={selectedMod}
>
	<svelte:fragment slot="details">
		{#if selectedMod && isOutdated(selectedMod)}
			<Button.Root
				class="bg-accent-600 hover:bg-accent-500 mt-2 flex w-full items-center justify-center gap-2 rounded-lg py-2 text-lg font-medium"
				on:click={() => updateMod(selectedMod)}
			>
				<Icon icon="mdi:arrow-up-circle" class="align-middle text-xl" />
				Update to {selectedMod?.versions[0].name}
			</Button.Root>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="banner">
		{#if unknownMods.length > 0}
			<div class="mr-3 mb-1 flex items-center rounded-lg bg-red-600 py-1.5 pr-1 pl-3 text-red-100">
				<Icon icon="mdi:alert-circle" class="mr-2 text-xl" />
				The following {unknownMods.length === 1 ? 'mod' : 'mods'} could not be found: {unknownMods
					.map((mod) => mod.fullName)
					.join(', ')}.
				<Button.Root
					class="ml-1 font-semibold text-white hover:text-red-100 hover:underline"
					on:click={() => {
						unknownMods.forEach(uninstall);
					}}
				>
					Uninstall them?
				</Button.Root>
			</div>
		{/if}

		<UpdateAllBanner {updates} />
	</svelte:fragment>

	<svelte:fragment slot="placeholder">
		{#if hasRefreshed}
			{#if totalModCount === 0}
				<span class="text-lg">No mods installed</span>
				<br />
				<a href="/browse" class="text-accent-400 hover:text-accent-300 hover:underline"
					>Click to browse Thunderstore</a
				>
			{:else}
				<span class="text-lg">No matching mods found in profile</span>
				<br />
				<span class="text-slate-400">Try to adjust your search query/filters</span>
			{/if}
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="item" let:data>
		<ProfileModListItem
			{...data}
			{reorderable}
			on:dragstart={onDragStart}
			on:dragover={onDragOver}
			on:dragend={onDragEnd}
			on:toggle={({ detail: newState }) => toggleMod(data.mod, newState)}
			on:click={() => modList.selectMod(data.mod)}
		/>
	</svelte:fragment>
</ModList>

<Popup title="Dependants of {activeMod?.name}" bind:open={dependantsOpen}>
	<div class="mt-4 text-center text-slate-300">
		{#if dependants.length === 0}
			No dependants found 😢
		{:else}
			<ModCardList names={dependants} showVersion={false} />
		{/if}
	</div>
</Popup>

<DependantsPopup
	bind:this={removeDependants}
	title="Confirm uninstallation"
	verb="Uninstall"
	description="The following mods depend on %s and will likely not work if it is uninstalled:"
	commandName="remove_mod"
	onExecute={() => {
		refreshProfiles();
		selectedMod = null;
	}}
	onCancel={refresh}
/>

<DependantsPopup
	bind:this={disableDependants}
	title="Confirm disabling"
	verb="Disable"
	description="The following mods depend on %s and will likely not work if it is disabled:"
	commandName="toggle_mod"
	onExecute={refreshProfiles}
	onCancel={refresh}
/>

<DependantsPopup
	bind:this={enableDependencies}
	title="Confirm enabling"
	verb="Enable"
	description="%s depends on the following disabled mods, and will likely not work if any of them are disabled:"
	commandName="toggle_mod"
	onExecute={refreshProfiles}
	onCancel={refresh}
	positive
/>
