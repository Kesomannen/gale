<script lang="ts">
	import * as api from '$lib/api';
	import DependantsPopup from '$lib/components/menu/DependantsPopup.svelte';
	import type {
		Mod,
		ModActionResponse,
		ProfileQuery,
		AvailableUpdate,
		Dependant,
		ModContextItem,
		SortBy
	} from '$lib/types';
	import ModList from '$lib/components/modlist/ModList.svelte';
	import {
		activeProfile,
		activeProfileLocked,
		profileQuery,
		refreshProfiles
	} from '$lib/stores.svelte';
	import { isOutdated } from '$lib/util';
	import Icon from '@iconify/svelte';
	import Popup from '$lib/components/ui/Popup.svelte';
	import ModCardList from '$lib/components/modlist/ModCardList.svelte';
	import ProfileModListItem from '$lib/components/modlist/ProfileModListItem.svelte';
	import UpdateAllBanner from '$lib/components/modlist/UpdateAllBanner.svelte';
	import { emit } from '@tauri-apps/api/event';
	import ProfileLockedBanner from '$lib/components/modlist/ProfileLockedBanner.svelte';
	import { defaultContextItems } from '$lib/context';
	import ModDetails from '$lib/components/modlist/ModDetails.svelte';
	import ModListFilters from '$lib/components/modlist/ModListFilters.svelte';
	import UnknownModsBanner from '$lib/components/modlist/UnknownModsBanner.svelte';

	const sortOptions: SortBy[] = [
		'custom',
		'installDate',
		'lastUpdated',
		'newest',
		'diskSpace',
		'name',
		'author',
		'rating',
		'downloads'
	];

	const contextItems: ModContextItem[] = [
		{
			label: 'Uninstall',
			icon: 'mdi:delete',
			onclick: (mod) =>
				uninstall({
					uuid: mod.uuid,
					fullName: mod.name
				}),
			showFor: (_, profileLocked) => !profileLocked
		},
		{
			label: 'Change version',
			icon: 'mdi:edit',
			onclick: () => {},
			showFor: (mod, profileLocked) => mod.versions.length > 1 && !profileLocked,
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
			onclick: (mod) => api.profile.openModDir(mod.uuid)
		},
		...defaultContextItems
	];

	let mods: Mod[] = $state([]);
	let totalModCount = $state(0);
	let unknownMods: Dependant[] = $state([]);
	let updates: AvailableUpdate[] = $state([]);

	let modList: ModList;
	let maxCount: number = $state(20);
	let selectedMod: Mod | null = $state(null);

	let removeDependants: DependantsPopup;
	let disableDependants: DependantsPopup;
	let enableDependencies: DependantsPopup;

	let dependantsOpen = $state(false);
	let dependants: string[] = $state([]);

	let activeMod: Mod | null = $state(null);

	let hasRefreshed = $state(false);
	let refreshing = false;

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		let result = await api.profile.query({ ...$profileQuery, maxCount });

		mods = result.mods;
		totalModCount = result.totalModCount;
		unknownMods = result.unknownMods;
		updates = result.updates;

		refreshing = false;
		hasRefreshed = true;
	}

	async function toggleMod(mod: Mod, newState: boolean) {
		let response = await api.profile.toggleMod(mod.uuid);

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
		let response = await api.profile.removeMod(mod.uuid);

		if (response.type == 'done') {
			selectedMod = null;
			await refreshProfiles();
		} else {
			removeDependants.openFor(mod, response.dependants);
		}
	}

	async function openDependants(mod: Mod) {
		dependants = await api.profile.getDependants(mod.uuid);

		activeMod = mod;
		dependantsOpen = true;
	}

	async function updateMod(mod: Mod | null, versionUuid?: string) {
		if (mod === null) return;

		if (versionUuid === undefined) {
			await api.profile.update.mods([mod.uuid], false);
		} else {
			await api.profile.update.changeModVersion({
				packageUuid: mod.uuid,
				versionUuid: versionUuid
			});
		}

		await refresh();

		if (selectedMod !== null) {
			selectedMod = mods.find((mod) => mod.uuid === selectedMod!.uuid) ?? null;
		}
	}

	let reorderUuid: string;
	let reorderPrevIndex: number;

	function ondragstart(evt: DragEvent) {
		if (!isDragApplicable(evt)) return;

		let element = evt.currentTarget as HTMLElement;

		reorderUuid = element.dataset.uuid!;
		reorderPrevIndex = parseInt(element.dataset.index!);

		evt.dataTransfer!.effectAllowed = 'move';
		evt.dataTransfer!.setData('text/html', element.outerHTML);
	}

	async function ondragover(evt: DragEvent) {
		if (!isDragApplicable(evt)) return;

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

		if ($profileQuery.sortOrder === 'descending') {
			delta *= -1; // list is reversed
		}

		await emit('reorder_mod', { uuid: reorderUuid, delta });
	}

	async function ondragend(evt: DragEvent) {
		if (!isDragApplicable(evt)) return;
		await emit('finish_reorder');
	}

	function isDragApplicable(evt: DragEvent) {
		if (!reorderable || evt.dataTransfer === null) return false;
		let items = [...evt.dataTransfer.items];
		return items.length === 0 || items[0].kind !== 'file';
	}

	$effect(() => {
		if (maxCount > 0) {
			$activeProfile;
			$profileQuery;
			refresh();
		}
	});

	let reorderable = $derived(
		$profileQuery.sortBy === 'custom' &&
			$profileQuery.searchTerm === '' &&
			$profileQuery.excludeCategories.length === 0 &&
			$profileQuery.includeCategories.length === 0 &&
			$profileQuery.includeDeprecated &&
			$profileQuery.includeNsfw &&
			$profileQuery.includeDisabled
	);

	let locked = $derived($activeProfileLocked);
</script>

<div class="flex grow overflow-hidden">
	<div class="flex w-[60%] grow flex-col overflow-hidden pt-3 pl-3">
		<ModListFilters {sortOptions} queryArgs={profileQuery} />

		{#if locked}
			<ProfileLockedBanner class="mr-4 mb-1" />
		{:else}
			<UpdateAllBanner {updates} />
		{/if}

		{#if unknownMods.length > 0}
			<UnknownModsBanner mods={unknownMods} {uninstall} />
		{/if}

		<ModList
			{mods}
			queryArgs={profileQuery}
			bind:this={modList}
			bind:maxCount
			bind:selected={selectedMod}
		>
			{#snippet placeholder()}
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
						<span class="text-primary-400">Try to adjust your search query/filters</span>
					{/if}
				{/if}
			{/snippet}

			{#snippet item({ mod, index, isSelected })}
				<ProfileModListItem
					{mod}
					{index}
					{isSelected}
					{contextItems}
					{reorderable}
					{locked}
					{ondragstart}
					{ondragover}
					{ondragend}
					ontoggle={(newState) => toggleMod(mod, newState)}
					onclick={() => modList.selectMod(mod)}
				/>
			{/snippet}
		</ModList>
	</div>

	{#if selectedMod}
		<ModDetails {locked} mod={selectedMod} {contextItems} onclose={() => (selectedMod = null)}>
			{#if selectedMod && isOutdated(selectedMod) && !locked}
				<button
					class="bg-accent-600 hover:bg-accent-500 mt-2 flex w-full items-center justify-center gap-2 rounded-lg py-2 text-lg font-medium"
					onclick={() => updateMod(selectedMod)}
				>
					<Icon icon="mdi:arrow-up-circle" class="align-middle text-xl" />
					Update to {selectedMod?.versions[0].name}
				</button>
			{/if}
		</ModDetails>
	{/if}
</div>

<Popup title="Dependants of {activeMod?.name}" bind:open={dependantsOpen}>
	<div class="text-primary-300 mt-4 text-center">
		{#if dependants.length === 0}
			No dependants found
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
