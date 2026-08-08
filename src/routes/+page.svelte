<script lang="ts">
	import * as api from '$lib/api';
	import DependantsDialog from '$lib/components/dialogs/DependantsDialog.svelte';
	import type {
		Mod,
		AvailableUpdate,
		Dependant,
		ModContextItem,
		SortBy,
		DependantWithVersion,
		ProfileListItem,
		LayoutItem,
		Folder
	} from '$lib/types';
	import { isOutdated } from '$lib/util';
	import Icon from '@iconify/svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import ModCardList from '$lib/components/ui/ModCardList.svelte';
	import ProfileModCardWithContext from '$lib/components/profile/ProfileModCardWithContext.svelte';
	import UpdateAllBanner from '$lib/components/mod-list/UpdateAllBanner.svelte';
	import ProfileLockedBanner from '$lib/components/mod-list/ProfileLockedBanner.svelte';
	import { defaultContextItems } from '$lib/context';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import UnknownModsBanner from '$lib/components/mod-list/UnknownModsBanner.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { profileQuery } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import ProfileModList from '$lib/components/profile/ProfileModList.svelte';
	import HelpCard from '$lib/components/ui/HelpCard.svelte';
	import config from '$lib/state/config.svelte';
	import Button from '$lib/components/ui/Button.svelte';

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

	const modContextItems: ModContextItem<Mod>[] = [
		{
			label: m.page_modContextItem_uninstall(),
			icon: 'mdi:delete',
			onclick: (mod) =>
				uninstall({
					uuid: mod.uuid,
					fullName: mod.name,
					backend: mod.backend
				}),
			showFor: (_, profileLocked) => !profileLocked
		},
		{
			label: m.page_modContextItem_changeVersion(),
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
			label: m.page_modContextItem_showDependants(),
			icon: 'mdi:source-branch',
			onclick: openDependants
		},
		{
			label: m.page_modContextItem_openFolder(),
			icon: 'mdi:folder',
			onclick: (mod) => api.profile.openModDir(mod.uuid)
		},
		{
			label: m.modDetails_editConfig(),
			icon: 'mdi:file-cog',
			showFor: (mod) => mod.configFile != null,
			onclick: (mod) => config.gotoModConfig(mod.configFile!)
		},
		...defaultContextItems
	];

	const folderContextItems: ModContextItem<Folder>[] = [
		{
			label: 'Rename',
			icon: 'mdi:pencil',
			onclick: (folder) => {},
			showFor: (_, profileLocked) => !profileLocked
		},
		{
			label: 'Delete',
			icon: 'mdi:minus',
			onclick: (folder) => onDeleteFolder(folder.id),
			showFor: (_, profileLocked) => !profileLocked
		},
		{
			label: 'Uninstall all',
			icon: 'mdi:delete',
			onclick: (folder) => {},
			showFor: (_, profileLocked) => !profileLocked
		}
	];

	let items: ProfileListItem[] = $state([]);
	let totalModCount = $state(0);
	let unknownMods: Dependant[] = $state([]);
	let updates: AvailableUpdate[] = $state([]);

	let selectedMod: Mod | null = $state(null);

	let removeDependants: DependantsDialog;
	let disableDependants: DependantsDialog;
	let enableDependencies: DependantsDialog;

	let dependantsOpen = $state(false);
	let dependants: DependantWithVersion[] = $state([]);

	let activeMod: Mod | null = $state(null);

	let hasRefreshed = $state(false);
	let refreshing = false;

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		console.log('Refreshing profile mod list');

		let result = await api.profile.query({ ...profileQuery.current, maxCount: null });

		// preserve the expanded/collapsed state of folders across refreshes
		const prevExpanded = new Map(
			items
				.filter((item) => item.type === 'folder')
				.map((item) => [item.folder.id, item.folder.isExpanded])
		);

		// if the list is filtered in some way, show the flat mod list,
		// otherwise show the custom layout with folders
		items = reorderable
			? buildItems(result.mods, result.layout, prevExpanded)
			: result.mods.map((mod) => ({ type: 'mod', mod }));
		totalModCount = result.totalModCount;
		unknownMods = result.unknownMods;
		updates = result.updates;

		refreshing = false;
		hasRefreshed = true;
	}

	/// Rebuilds the displayed `ListItem[]` from the query result and the profile layout.
	function buildItems(
		mods: Mod[],
		layout: LayoutItem[],
		prevExpanded: Map<string, boolean>
	): ProfileListItem[] {
		const modByUuid = new Map(mods.map((mod) => [mod.uuid, mod]));

		const items: ProfileListItem[] = [];
		for (const item of layout) {
			// resolve the uuids in the layout to actual mods
			if (item.type === 'mod') {
				const mod = modByUuid.get(item.uuid);
				if (mod) items.push({ type: 'mod', mod });
				continue;
			}

			const folderMods = item.mods
				.map((uuid) => modByUuid.get(uuid))
				.filter((mod) => mod !== undefined);

			items.push({
				type: 'folder',
				folder: {
					id: item.id,
					name: item.name,
					mods: folderMods,
					isExpanded: prevExpanded.get(item.id) ?? false
				}
			});
		}

		return items;
	}

	/// Serializes the currently displayed (display order) items into the canonical
	/// layout stored by the backend, accounting for a reversed custom sort order.
	function serializeLayout(displayItems: ProfileListItem[]): LayoutItem[] {
		return displayItems.map((item) =>
			item.type === 'mod'
				? { type: 'mod', uuid: item.mod.uuid }
				: {
						type: 'folder',
						id: item.folder.id,
						name: item.folder.name,
						mods: item.folder.mods.map((mod) => mod.uuid)
					}
		);
	}

	async function onLayoutChange(newItems: ProfileListItem[]) {
		await api.profile.setProfileLayout(serializeLayout(newItems));
	}

	async function onFolderToggle(folderId: string, newState: boolean) {
		await api.profile.setFolderModsState(folderId, newState);
		await refresh();
	}

	async function onFolderRename(folderId: string, name: string) {
		const newItems = items.map((item) =>
			item.type === 'folder' && item.folder.id === folderId
				? { ...item, folder: { ...item.folder, name } }
				: item
		);

		await onLayoutChange(newItems);
	}

	async function onDeleteFolder(folderId: string) {
		const newItems: ProfileListItem[] = [];
		for (const item of items) {
			if (item.type === 'folder' && item.folder.id === folderId) {
				// the folder's mods become loose, keeping their position
				newItems.push(...item.folder.mods.map((mod): ProfileListItem => ({ type: 'mod', mod })));
			} else {
				newItems.push(item);
			}
		}

		await onLayoutChange(newItems);
	}

	async function toggleMod(mod: Mod, newState: boolean) {
		mod.enabled = !mod.enabled;
		let response = await api.profile.toggleMod(mod.uuid);

		if (response.type == 'done') {
			await refresh();
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
			refresh();
		} else {
			removeDependants.openFor(mod, response.dependants);
		}
	}

	async function forceUninstall(mod: Dependant) {
		await api.profile.forceRemoveMods([mod.uuid]);
		selectedMod = null;
	}

	async function openDependants(mod: Mod) {
		dependants = (await api.profile.getDependants(mod.uuid)).map((d) => ({
			backend: mod.backend,
			...d
		}));

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
				versionUuid: versionUuid,
				backend: mod.backend
			});
		}

		await refresh();

		if (selectedMod === null) return;

		const newItem = items.find(
			(item) => item.type === 'mod' && item.mod.uuid === selectedMod!.uuid
		);

		if (!newItem || newItem.type !== 'mod') {
			return;
		}

		selectedMod = newItem.mod;
	}

	$effect(() => {
		profiles.active;
		profileQuery.current;
		refresh();
	});

	let reorderable = $derived(
		profileQuery.current.sortBy === 'custom' &&
			profileQuery.current.sortOrder === 'ascending' &&
			profileQuery.current.searchTerm === '' &&
			profileQuery.current.excludeCategories.length === 0 &&
			profileQuery.current.includeCategories.length === 0 &&
			profileQuery.current.includeDeprecated &&
			profileQuery.current.includeNsfw &&
			profileQuery.current.includeDisabled
	);

	let locked = $derived(profiles.activeLocked);
</script>

<div class="flex grow">
	<div class="flex w-[60%] grow flex-col px-4 pt-4">
		<ModListFilters {sortOptions} queryArgs={profileQuery.current} />

		{#if locked}
			<ProfileLockedBanner class="mb-1" />
		{:else}
			<UpdateAllBanner {updates} />
		{/if}

		{#if unknownMods.length > 0}
			<UnknownModsBanner mods={unknownMods} uninstall={forceUninstall} />
		{/if}

		{#if items.length === 0 && hasRefreshed}
			{#if totalModCount === 0}
				<HelpCard icon="ph:ghost" title={m.page_modList_noMods_1()}>
					<a href="/browse" class="text-accent-400 hover:text-accent-300 hover:underline"
						><Icon
							icon="mdi:store-search"
							class="mr-0.5 ml-1  inline"
							inline
						/>{m.page_modList_noMods_2()}</a
					>
				</HelpCard>
			{:else}
				<HelpCard class="mt-4" title={m.page_modList_noResults_1()} icon="mdi:magnify">
					{m.page_modList_noResults_2()}
				</HelpCard>
			{/if}
		{:else}
			<ProfileModList
				bind:items
				bind:selectedMod
				{locked}
				{reorderable}
				{modContextItems}
				{folderContextItems}
				onModToggle={(mod, newState) => toggleMod(mod, newState)}
				onLayoutChange={() => onLayoutChange(items)}
				{onFolderToggle}
			/>
		{/if}
	</div>

	{#if selectedMod}
		<ModDetails
			{locked}
			mod={selectedMod}
			contextItems={modContextItems}
			onclose={() => (selectedMod = null)}
		>
			{#if isOutdated(selectedMod) && !locked}
				<Button
					color="accent"
					size="lg"
					icon="mdi:arrow-up-circle"
					class="mt-2"
					onclick={() => updateMod(selectedMod)}
				>
					{m.page_modDetails_button({ version: selectedMod.versions[0].name })}
				</Button>
			{/if}
		</ModDetails>
	{/if}
</div>

<Dialog
	title={m.page_dialog_title({ name: activeMod?.name ?? m.unknown() })}
	bind:open={dependantsOpen}
>
	<div class="text-primary-300 mt-4 text-center">
		{#if dependants.length === 0}
			{m.page_dialog_noDependants()}
		{:else}
			<ModCardList mods={dependants} showVersion={false}>
				{#snippet cardChildren({ mod })}
					{#if mod.preferredVersion}
						<div class="text-primary-400">
							Preferred Version: {mod.preferredVersion}
						</div>
					{/if}
				{/snippet}
			</ModCardList>
		{/if}
	</div>
</Dialog>

<DependantsDialog
	bind:this={removeDependants}
	title={m.page_dependantsDialog_uninstall_title()}
	verb={m.page_dependantsDialog_uninstall_verb()}
	description={m.page_dependantsDialog_uninstall_description()}
	commandName="remove_mod"
	onExecute={() => {
		selectedMod = null;
		refresh();
	}}
	onCancel={refresh}
/>

<DependantsDialog
	bind:this={disableDependants}
	title={m.page_dependantsDialog_disable_title()}
	verb={m.page_dependantsDialog_disable_verb()}
	description={m.page_dependantsDialog_disable_description()}
	commandName="toggle_mod"
	onCancel={refresh}
	onExecute={() => {
		refresh();
	}}
/>

<DependantsDialog
	bind:this={enableDependencies}
	title={m.page_dependantsDialog_enable_title()}
	verb={m.page_dependantsDialog_enable_verb()}
	description={m.page_dependantsDialog_enable_description()}
	commandName="toggle_mod"
	onCancel={refresh}
	positive
	onExecute={() => {
		refresh();
	}}
/>
