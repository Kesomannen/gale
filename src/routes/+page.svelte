<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import DependantsPopup from '$lib/menu/DependantsPopup.svelte';
	import {
		type Mod,
		type ModActionResponse,
		type ProfileQuery,
		type AvailableUpdate,
		SortBy,
		type Dependant
	} from '$lib/models';
	import ModContextMenuItem from '$lib/modlist/ModContextMenuItem.svelte';
	import ModList from '$lib/modlist/ModList.svelte';
	import { activeProfile, profileQuery, refreshProfiles } from '$lib/stores';
	import { isOutdated } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Button, DropdownMenu } from 'bits-ui';
	import { fly } from 'svelte/transition';
	import Popup from '$lib/components/Popup.svelte';
	import ModCardList from '$lib/modlist/ModCardList.svelte';
	import ProfileModListItem from '$lib/modlist/ProfileModListItem.svelte';
	import UpdateAllBanner from '$lib/modlist/UpdateAllBanner.svelte';

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

	let mods: Mod[] = [];
	let unknownMods: Dependant[] = [];
	let updates: AvailableUpdate[] = [];
	let activeMod: Mod | null = null;

	let removeDependants: DependantsPopup;
	let disableDependants: DependantsPopup;
	let enableDependencies: DependantsPopup;

	let dependantsOpen = false;
	let dependants: string[] | null;

	$: {
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

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		let result = await invokeCommand<ProfileQuery>('query_profile', { args: $profileQuery });

		mods = result.mods;
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
			activeMod = null;
			await refreshProfiles();
		} else {
			removeDependants.openFor(mod, response.dependants);
		}
	}

	async function openDependants() {
		if (!activeMod) return;

		dependants = null;
		dependantsOpen = true;

		dependants = await invokeCommand<string[]>('get_dependants', {
			uuid: activeMod.uuid
		});
	}

	async function updateActiveMod(version: 'latest' | { specific: string }) {
		if (!activeMod) return;

		if (version === 'latest') {
			await invokeCommand('update_mods', { uuids: [activeMod.uuid], respectIgnored: false });
		} else {
			await invokeCommand('change_mod_version', {
				modRef: {
					packageUuid: activeMod.uuid,
					versionUuid: version.specific
				}
			});
		}

		await refresh();

		activeMod = mods.find((mod) => mod.uuid === activeMod!.uuid) ?? null;
	}

	let dragElement: HTMLElement | null;
	let dragStartIndex: number;
	let dragTargetIndex: number;

	function onDragStart(evt: DragEvent) {
		if (!reorderable || !evt.dataTransfer) return;

		dragElement = evt.currentTarget as HTMLElement;

		dragStartIndex = parseInt(dragElement.dataset.index!);
		dragTargetIndex = dragStartIndex;

		evt.dataTransfer.effectAllowed = 'move';
		evt.dataTransfer.setData('text/html', dragElement.outerHTML);

		console.log('started dragging mod at', dragStartIndex);
	}

	function onDragOver(evt: DragEvent) {
		if (!reorderable || !dragElement) return;

		let target = evt.currentTarget as HTMLElement;
		target.dataset.dragTarget = 'true';

		let targetIndex = parseInt(target.dataset.index!);

		dragTargetIndex = targetIndex;
		console.log('new drag target:', dragTargetIndex);
	}

	async function onDragEnd(evt: DragEvent) {
		console.log('onDragEnd:', evt);
		if (!reorderable || !dragElement) return;

		let uuid = dragElement.dataset.uuid!;
		let delta = dragTargetIndex - dragStartIndex;
		dragElement = null;

		console.log('moving', dragStartIndex, 'to', dragTargetIndex);

		if (delta === 0) {
			return;
		}

		await invokeCommand('reorder_mod', { uuid, delta });

		await refresh();
	}
</script>

<ModList {sortOptions} bind:mods bind:activeMod queryArgs={profileQuery}>
	<svelte:fragment slot="details">
		{#if activeMod && isOutdated(activeMod)}
			<Button.Root
				class="mt-2 flex w-full items-center justify-center gap-2 rounded-lg bg-green-600 py-2 text-lg font-medium hover:bg-green-500"
				on:click={() => updateActiveMod('latest')}
			>
				<Icon icon="mdi:arrow-up-circle" class="align-middle text-xl" />
				Update to {activeMod?.versions[0].name}
			</Button.Root>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="context">
		{#if activeMod && activeMod?.versions.length > 1}
			<DropdownMenu.Sub>
				<DropdownMenu.SubTrigger
					class="flex cursor-default items-center truncate rounded-md py-1 pl-3 pr-1 text-left text-slate-300 hover:bg-gray-600 hover:text-slate-100"
				>
					<Icon class="mr-1.5 text-lg" icon="mdi:edit" />
					Change version
					<Icon class="ml-4 text-xl" icon="mdi:chevron-right" />
				</DropdownMenu.SubTrigger>
				<DropdownMenu.SubContent
					class="mr-2 flex max-h-96 flex-col gap-0.5 overflow-y-auto rounded-lg border border-gray-500 bg-gray-700 p-1 shadow-xl"
					transition={fly}
					transitionConfig={{ duration: 50 }}
				>
					{#each activeMod?.versions ?? [] as version}
						<DropdownMenu.Item
							class="flex flex-shrink-0 cursor-default items-center truncate rounded-md py-1 pl-3 pr-12 text-left text-slate-300 hover:bg-gray-600 hover:text-slate-100"
							on:click={() => updateActiveMod({ specific: version.uuid })}
						>
							{version.name}
						</DropdownMenu.Item>
					{/each}
				</DropdownMenu.SubContent>
			</DropdownMenu.Sub>
		{/if}

		<ModContextMenuItem
			label="Uninstall"
			icon="mdi:delete"
			onClick={() =>
				uninstall({
					uuid: activeMod?.uuid ?? '',
					fullName: activeMod?.name ?? ''
				})}
		/>

		<ModContextMenuItem icon="mdi:source-branch" label="Show dependants" onClick={openDependants} />

		<ModContextMenuItem
			label="Open directory"
			icon="mdi:folder"
			onClick={() => invokeCommand('open_plugin_dir', { uuid: activeMod?.uuid })}
		/>
	</svelte:fragment>

	<svelte:fragment slot="banner">
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
	</svelte:fragment>

	<svelte:fragment slot="item" let:mod let:index let:isSelected>
		<ProfileModListItem
			{mod}
			{index}
			{isSelected}
			{reorderable}
			on:dragstart={onDragStart}
			on:dragend={onDragEnd}
			on:dragover={onDragOver}
			on:toggle={({ detail: newState }) => toggleMod(mod, newState)}
		/>
	</svelte:fragment>
</ModList>

<Popup title="Dependants of {activeMod?.name}" bind:open={dependantsOpen}>
	<div class="mt-4 text-center text-slate-300">
		{#if dependants}
			{#if dependants.length === 0}
				No dependants found 😢
			{:else}
				<ModCardList names={dependants} />
			{/if}
		{:else}
			Loading...
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
		activeMod = null;
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
