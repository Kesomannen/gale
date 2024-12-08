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

	import { dndzone } from 'svelte-dnd-action';
	import { onMount } from 'svelte';
	import { flip } from 'svelte/animate';
	import ModDetails from '$lib/modlist/ModDetails.svelte';
	import ModListFilters from '$lib/modlist/ModListFilters.svelte';

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
			label: 'Open directory',
			icon: 'mdi:folder',
			onclick: (mod) => invokeCommand('open_mod_dir', { uuid: mod.uuid })
		},
		{
			label: 'Open website',
			icon: 'mdi:open-in-new',
			onclick: (mod) => openIfNotNull(mod.websiteUrl),
			showFor: (mod) => mod.websiteUrl !== null && mod.websiteUrl.length > 0
		},
		{
			label: 'Donate',
			icon: 'mdi:heart',
			onclick: (mod) => openIfNotNull(mod.donateUrl),
			showFor: (mod) => mod.donateUrl !== null
		}
	];

	let mods: Mod[] = [];
	let unknownMods: Dependant[] = [];
	let updates: AvailableUpdate[] = [];

	let maxCount: number = 100;
	let selectedMod: Mod | null = null;
	let activeMod: Mod | null = null;

	let removeDependants: DependantsPopup;
	let disableDependants: DependantsPopup;
	let enableDependencies: DependantsPopup;

	let dependantsOpen = false;
	let dependants: string[];

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

	let refreshing = false;

	onMount(() => refresh());

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		let result = await invokeCommand<ProfileQuery>('query_profile', {
			args: { ...$profileQuery, maxCount }
		});

		mods = result.mods.map((mod, i) => ({ id: i, children: [], ...mod }));
		console.log(mods);
		unknownMods = result.unknownMods;
		updates = result.updates;

		refreshing = false;
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

	export function selectMod(mod: Mod) {
		if (selectedMod === null || selectedMod.uuid !== mod.uuid) {
			selectedMod = mod;
		} else {
			selectedMod = null;
		}
	}

	function openIfNotNull(url: string | null) {
		if (url !== null) open(url);
	}
</script>

<!--
<div
	class="overflow-y-scroll"
	use:dndzone={{ items: mods, flipDurationMs: 150 }}
	on:consider={({ detail }) => (mods = detail.items)}
	on:finalize={({ detail }) => (mods = detail.items)}
>
	{#each mods as mod (mod.id)}
		<div class="cursor-move bg-black px-4 py-2 text-white" animate:flip={{ duration: 150 }}>
			{mod.name}
		</div>
	{/each}
</div>
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
				class="mt-2 flex w-full items-center justify-center gap-2 rounded-lg bg-accent-600 py-2 text-lg font-medium hover:bg-accent-500"
				on:click={() => updateMod(selectedMod)}
			>
				<Icon icon="mdi:arrow-up-circle" class="align-middle text-xl" />
				Update to {selectedMod?.versions[0].name}
			</Button.Root>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="banner">

	</svelte:fragment>

	<svelte:fragment slot="item" let:data>
		<ProfileModListItem
			{...data}
			{reorderable}
			on:dragstart={onDragStart}
			on:dragover={onDragOver}
			on:toggle={({ detail: newState }) => toggleMod(data.mod, newState)}
			on:click={() => modList.selectMod(data.mod)}
		/>
	</svelte:fragment>
</ModList>
-->

<div class="flex flex-grow overflow-hidden">
	<div class="flex w-[60%] flex-grow flex-col overflow-hidden pl-3 pt-3">
		<ModListFilters {sortOptions} queryArgs={profileQuery} />

		{#if unknownMods.length > 0}
			<div class="mb-1 mr-3 flex items-center rounded-lg bg-red-600 py-1.5 pl-3 pr-1 text-red-100">
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

		{#if mods.length === 0}
			<div class="mt-4 text-center text-lg text-slate-300">No mods found 😥</div>
		{:else}
			<div
				class="overflow-y-scroll"
				use:dndzone={{ items: mods, flipDurationMs: 150 }}
				on:consider={({ detail }) => (mods = detail.items)}
				on:finalize={({ detail }) => (mods = detail.items)}
			>
				{#each mods as mod (mod.id)}
					<button
						class="w-full py-1 text-left text-white hover:bg-slate-600"
						animate:flip={{ duration: 150 }}
						on:click={() => selectMod(mod)}
						use:dndzone={{ items: mod.children, flipDurationMs: 150 }}
						on:consider={({ detail }) => (mod.children = detail.items)}
						on:finalize={({ detail }) => (mod.children = detail.items)}
					>
						<div>{mod.name}</div>
						<div class="pl-2">
							{#each mod.children as child (child.id)}
								{child}
							{/each}
						</div>
					</button>
				{/each}
			</div>
		{/if}
	</div>

	{#if selectedMod !== null}
		<ModDetails mod={selectedMod} {contextItems} on:close={() => (selectedMod = null)}>
			{#if selectedMod && isOutdated(selectedMod)}
				<Button.Root
					class="mt-2 flex w-full items-center justify-center gap-2 rounded-lg bg-accent-600 py-2 text-lg font-medium hover:bg-accent-500"
					on:click={() => updateMod(selectedMod)}
				>
					<Icon icon="mdi:arrow-up-circle" class="align-middle text-xl" />
					Update to {selectedMod?.versions[0].name}
				</Button.Root>
			{/if}
		</ModDetails>
	{/if}
</div>

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
