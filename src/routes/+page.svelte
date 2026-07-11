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
		ListItem
	} from '$lib/types';
	import { isOutdated } from '$lib/util';
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
	import { goto } from '$app/navigation';

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

	let mods: Mod[] = $state([]);
	let items: ListItem[] = $state([]);
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

		let result = await api.profile.query({ ...profileQuery.current, maxCount: null });

		mods = result.mods;
		items = result.mods.map((mod) => ({ type: 'mod', mod }));
		totalModCount = result.totalModCount;
		unknownMods = result.unknownMods;
		updates = result.updates;

		refreshing = false;
		hasRefreshed = true;
	}

	async function toggleMod(mod: Mod, newState: boolean) {
		mod.enabled = !mod.enabled;
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
		} else {
			removeDependants.openFor(mod, response.dependants);
		}
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

		if (selectedMod !== null) {
			selectedMod = mods.find((mod) => mod.uuid === selectedMod!.uuid) ?? null;
		}
	}

	async function onmove(item: ListItem, fromIndex: number, toIndex: number) {
		if (item.type !== 'mod') return;

		let delta = toIndex - fromIndex;

		if (profileQuery.current.sortOrder === 'descending') {
			delta *= -1; // list is reversed
		}

		await emit('reorder_mod', { uuid: item.mod.uuid, delta });
	}

	$effect(() => {
		profiles.active;
		profileQuery.current;
		refresh();
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
		<ModListFilters {sortOptions} queryArgs={profileQuery.current} />

		{#if locked}
			<ProfileLockedBanner class="mb-1" />
		{:else}
			<UpdateAllBanner {updates} />
		{/if}

		{#if unknownMods.length > 0}
			<UnknownModsBanner mods={unknownMods} {uninstall} />
		{/if}

		{#if mods.length === 0 && hasRefreshed}
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
				<HelpCard class="mt-8" title={m.page_modList_noResults_1()}>
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
						selected={selectedMod?.uuid === mod.uuid}
						ontoggle={(newState) => toggleMod(mod, newState)}
						onclick={() => {
							if (selectedMod?.uuid === mod.uuid) {
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
		<ModDetails {locked} mod={selectedMod} {contextItems} onclose={() => (selectedMod = null)}>
			{#if isOutdated(selectedMod) && !locked}
				<button
					class="bg-accent-700 hover:bg-accent-600 mt-2 flex w-full items-center justify-center gap-2 rounded-lg py-2 text-lg font-medium"
					onclick={() => updateMod(selectedMod)}
				>
					<Icon icon="mdi:arrow-up-circle" class="align-middle text-xl" />
					{m.page_modDetails_button({ version: selectedMod.versions[0].name })}
				</button>
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
