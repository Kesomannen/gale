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
		ListItem,
		Backend,
		ProfileMod,
		ModId
	} from '$lib/types';
	import {
		isNonReleaseVersion,
		mapModContextItem,
		resolveModContextItems,
		shouldWarnForeignDownload
	} from '$lib/util';
	import Icon from '@iconify/svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import ModCardList from '$lib/components/ui/ModCardList.svelte';
	import ProfileModListItem from '$lib/components/mod-list/ProfileModListItem.svelte';
	import UpdateAllBanner from '$lib/components/mod-list/UpdateAllBanner.svelte';
	import { emit } from '@tauri-apps/api/event';
	import ProfileLockedBanner from '$lib/components/mod-list/ProfileLockedBanner.svelte';
	import { defaultContextItems } from '$lib/context';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import UnknownModsBanner from '$lib/components/mod-list/UnknownModsBanner.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { profileQuery } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import ReorderableList from '$lib/components/profile/ReorderableList.svelte';
	import HelpCard from '$lib/components/ui/HelpCard.svelte';
	import config from '$lib/state/config.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { untrack } from 'svelte';
	import ForeignDownloadDialog from '$lib/components/dialogs/ForeignDownloadDialog.svelte';

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

	const contextItems: ModContextItem<ProfileMod>[] = [
		{
			label: m.page_modContextItem_uninstall(),
			icon: 'mdi:delete',
			onclick: (mod) => uninstall(mod),
			showFor: (_, profileLocked) => !profileLocked
		},
		{
			label: m.page_modContextItem_changeVersion(),
			icon: 'mdi:edit',
			onclick: () => {},
			showFor: (mod, profileLocked) => mod.data.versions.length > 1 && !profileLocked,
			children: (mod) =>
				mod.data.versions
					.filter((version) => version.uuid != mod.data.versionUuid)
					.map((version) => ({
						label: version.name,
						onclick: () => changeModVersion(mod, { versionUuid: version.uuid })
					}))
		},
		{
			label: 'Change source',
			icon: 'mdi:web',
			onclick: () => {},
			showFor: (mod, profileLocked) => mod.alternateBackend !== null && !profileLocked,
			children: (mod) => {
				if (!mod.alternateBackend) return [];

				const { backend, latestVersion, latestVersionUuid } = mod.alternateBackend;

				return [
					{
						label: `${backend} (${latestVersion})`,
						onclick: () => changeModVersion(mod, { backend, versionUuid: latestVersionUuid })
					}
				];
			}
		},
		{
			label: m.page_modContextItem_showDependants(),
			icon: 'mdi:source-branch',
			onclick: openDependants
		},
		{
			label: m.page_modContextItem_openFolder(),
			icon: 'mdi:folder',
			onclick: (mod) => api.profile.openModDir(mod.data.uuid)
		},
		{
			label: m.modDetails_editConfig(),
			icon: 'mdi:file-cog',
			showFor: (mod) => mod.configFile != null,
			onclick: (mod) => config.gotoModConfig(mod.configFile!)
		},
		...defaultContextItems.map((item) =>
			mapModContextItem<Mod, ProfileMod>(item, (mod) => mod.data)
		)
	];

	let mods: ProfileMod[] = $state([]);
	let items: ListItem[] = $state([]);
	let totalModCount = $state(0);
	let unknownMods: Dependant[] = $state([]);
	// map from package uuids to updates
	let updates: Map<string, AvailableUpdate> = $state(new Map());

	let selectedMod: ProfileMod | null = $state(null);

	let removeDependants: DependantsDialog;
	let disableDependants: DependantsDialog;
	let enableDependencies: DependantsDialog;

	let dependantsOpen = $state(false);
	let dependants: DependantWithVersion[] = $state([]);

	let activeMod: ProfileMod | null = $state(null);

	let hasRefreshed = $state(false);

	let refreshPromise: Promise<void> | null = $state(null);

	let foreignDownloadDialogOpen = $state(false);

	async function refresh() {
		if (refreshPromise !== null) {
			// make sure if this function is awaited while already refreshing, we wait until
			// the refresh is done before returning so the caller sees the fresh values
			await refreshPromise;
			return;
		}

		refreshPromise = (async () => {
			const result = await api.profile.query({ ...profileQuery.current, maxCount: null });

			const updateMap = new Map(
				result.updates.map((update) => [update.updatedId.packageUuid, update])
			);

			mods = result.mods;
			items = result.mods.map((mod) => ({ type: 'mod', mod }));
			totalModCount = result.totalModCount;
			unknownMods = result.unknownMods;
			updates = updateMap;

			if (selectedMod !== null) {
				selectedMod = mods.find((mod) => mod.data.uuid === selectedMod!.data.uuid) ?? null;
			}

			hasRefreshed = true;
		})();

		await refreshPromise;
		refreshPromise = null;
	}

	async function toggleMod(mod: ProfileMod, newState: boolean) {
		mod.enabled = !mod.enabled;
		let response = await api.profile.toggleMod(mod.data.uuid);

		if (response.type == 'done') {
			refresh();
			return;
		}

		if (newState) {
			enableDependencies.openFor(mod.data, response.dependants);
		} else {
			disableDependants.openFor(mod.data, response.dependants);
		}
	}

	async function uninstall(mod: ProfileMod) {
		let response = await api.profile.removeMod(mod.data.uuid);

		if (response.type == 'done') {
			selectedMod = null;
		} else {
			removeDependants.openFor(mod.data, response.dependants);
		}
	}

	async function forceUninstall(...uuids: string[]) {
		await api.profile.forceRemoveMods(uuids);
		selectedMod = null;
	}

	async function openDependants(mod: ProfileMod) {
		dependants = (await api.profile.getDependants(mod.data.uuid)).map((d) => ({
			backend: mod.data.backend,
			...d
		}));

		activeMod = mod;
		dependantsOpen = true;
	}

	async function changeModVersion(
		mod: ProfileMod,
		opts: { versionUuid?: string; backend?: Backend }
	) {
		await api.profile.update.changeModVersion({
			packageUuid: mod.data.uuid,
			versionUuid: opts.versionUuid ?? mod.data.versionUuid,
			backend: opts.backend ?? mod.data.backend
		});
		await refresh();
	}

	async function updateModToLatest(mod: ProfileMod) {
		await api.profile.update.mods([mod.data.uuid], false);
		await refresh();
	}

	async function onmove(item: ListItem, fromIndex: number, toIndex: number) {
		if (item.type !== 'mod') return;

		let delta = toIndex - fromIndex;

		if (profileQuery.current.sortOrder === 'descending') {
			delta *= -1; // list is reversed
		}

		await emit('reorder_mod', { uuid: item.mod.data.uuid, delta });
	}

	$effect(() => {
		profiles.active;
		// read all fields of profileQuery.current to trigger the effect when any of them change
		JSON.stringify(profileQuery.current);
		untrack(() => refresh());
	});

	let reorderable = $derived(
		profileQuery.current.sortBy === 'custom' &&
			profileQuery.current.searchTerm === '' &&
			profileQuery.current.excludeCategories.length === 0 &&
			profileQuery.current.includeCategories.length === 0 &&
			profileQuery.current.includeDeprecated &&
			profileQuery.current.includeNsfw &&
			profileQuery.current.includeDisabled
	);

	let locked = $derived(profiles.activeLocked);
</script>

<div class="flex grow overflow-hidden">
	<div class="flex w-[60%] grow flex-col overflow-hidden px-4 pt-4">
		<ModListFilters {sortOptions} bind:queryArgs={profileQuery.current} />

		{#if locked}
			<ProfileLockedBanner class="mb-1" />
		{:else}
			<UpdateAllBanner updates={updates.values().toArray()} />
		{/if}

		{#if unknownMods.length > 0}
			<UnknownModsBanner mods={unknownMods} uninstallAll={forceUninstall} />
		{/if}

		{#if mods.length === 0 && hasRefreshed}
			{#if totalModCount === 0}
				<HelpCard icon="ph:ghost" title={m.page_modList_noMods_1()}>
					<a
						href="/browse"
						class="text-accent-600 hover:text-accent-700 dark:text-accent-400 dark:hover:text-accent-300 hover:underline"
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
			<ReorderableList bind:items {onmove} {reorderable}>
				{#snippet mod({ mod, index })}
					<ProfileModListItem
						{mod}
						{index}
						{locked}
						{contextItems}
						update={updates.get(mod.data.uuid)}
						selected={selectedMod?.data.uuid === mod.data.uuid}
						ontoggle={(newState) => toggleMod(mod, newState)}
						onclick={() => {
							if (selectedMod?.data.uuid === mod.data.uuid) {
								selectedMod = null;
							} else {
								selectedMod = mod;
							}
						}}
					/>
				{/snippet}
			</ReorderableList>
		{/if}
	</div>

	{#if selectedMod}
		{@const update = updates.get(selectedMod.data.uuid)}

		<ModDetails
			mod={selectedMod.data}
			contextItems={resolveModContextItems(contextItems, selectedMod, locked)}
			onclose={() => (selectedMod = null)}
		>
			{#if update && !locked}
				{@const isPrerelease = isNonReleaseVersion(update?.new)}

				<Button
					color={isPrerelease ? 'primary' : 'accent'}
					icon={isPrerelease ? 'mdi:flask-outline' : 'mdi:arrow-up-circle'}
					size="lg"
					class="mt-2"
					onclick={async () => {
						const prefs = await api.prefs.get();
						if (shouldWarnForeignDownload(update.updatedId, prefs)) {
							foreignDownloadDialogOpen = true;
						} else {
							updateModToLatest(selectedMod!);
						}
					}}
				>
					{m.page_modDetails_button({ version: update.new })}
				</Button>
			{/if}
		</ModDetails>
	{/if}
</div>

<Dialog
	title={m.page_dialog_title({ name: activeMod?.data.name ?? m.unknown() })}
	bind:open={dependantsOpen}
>
	<div class="text-primary-600 dark:text-primary-300 mt-4 text-center">
		{#if dependants.length === 0}
			{m.page_dialog_noDependants()}
		{:else}
			<ModCardList mods={dependants} showVersion={false}>
				{#snippet cardChildren({ mod })}
					{#if mod.preferredVersion}
						<div class="text-primary-500 dark:text-primary-400">
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
/>

<DependantsDialog
	bind:this={enableDependencies}
	title={m.page_dependantsDialog_enable_title()}
	verb={m.page_dependantsDialog_enable_verb()}
	description={m.page_dependantsDialog_enable_description()}
	commandName="toggle_mod"
	onCancel={refresh}
	positive
/>

<ForeignDownloadDialog
	bind:open={foreignDownloadDialogOpen}
	onConfirm={() => updateModToLatest(selectedMod!)}
/>
