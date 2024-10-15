<script lang="ts" context="module">
	import { writable } from 'svelte/store';

	const updateBannerThreshold = writable(0);
</script>

<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import DependantsPopup from '$lib/menu/DependantsPopup.svelte';
	import {
		type Mod,
		type ModActionResponse,
		type ProfileQuery,
		type AvailableUpdate,
		SortBy,
		SortOrder,
		type Dependant
	} from '$lib/models';
	import ModDetailsDropdownItem from '$lib/modlist/ModDetailsDropdownItem.svelte';
	import ModList from '$lib/modlist/ModList.svelte';
	import { activeProfile, profileQuery, refreshProfiles } from '$lib/stores';
	import { isBefore, isOutdated } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Button, DropdownMenu, Switch } from 'bits-ui';
	import { fly } from 'svelte/transition';
	import Popup from '$lib/components/Popup.svelte';
	import { onMount } from 'svelte';
	import ModCardList from '$lib/modlist/ModCardList.svelte';
	import ModCard from '$lib/modlist/ModCard.svelte';
	import Checklist from '$lib/components/Checklist.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import ProfileModListItem from '$lib/modlist/ProfileModListItem.svelte';

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
	let allUpdates: AvailableUpdate[] = [];
	let activeMod: Mod | undefined;

	let removeDependants: DependantsPopup;
	let disableDependants: DependantsPopup;
	let enableDependencies: DependantsPopup;

	let updateAllOpen = false;
	let dependantsOpen = false;
	let dependants: string[] | null;

	let includeUpdates: Map<AvailableUpdate, boolean> = new Map();

	$: updates = allUpdates.filter((update) => !update.ignore);

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

	// really ugly hack because reactive statements run once immediately no matter what :/
	let isFirst = true;

	$: {
		$activeProfile;
		resetBannerThreshold();
	}

	function resetBannerThreshold() {
		if (isFirst) {
			isFirst = false;
			return;
		}

		$updateBannerThreshold = 0;
	}

	async function refresh() {
		let result = await invokeCommand<ProfileQuery>('query_profile', { args: $profileQuery });
		mods = result.mods;
		unknownMods = result.unknownMods;
		allUpdates = result.updates;
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
			enableDependencies.openFor(mod, response.content);
		} else {
			disableDependants.openFor(mod, response.content);
		}
	}

	async function uninstall(mod: Dependant) {
		let response = await invokeCommand<ModActionResponse>('remove_mod', { uuid: mod.uuid });

		if (response.type == 'done') {
			activeMod = undefined;
			await refreshProfiles();
			return;
		}

		removeDependants.openFor(mod, response.content);
	}

	async function onReorder(uuid: string, delta: number) {
		let oldIndex = mods.findIndex((mod) => mod.uuid === uuid);

		if (oldIndex === -1) {
			console.warn('Could not find mod with uuid', uuid);
			return;
		}

		let newIndex = oldIndex + delta;

		if (newIndex < 0 || newIndex >= mods.length) return;
		let temp = mods[newIndex];
		mods[newIndex] = mods[oldIndex];
		mods[oldIndex] = temp;
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

		activeMod = mods.find((mod) => mod.uuid === activeMod!.uuid);
	}

	let dragElement: HTMLElement | null;
	let totalDelta = 0;

	function onDragStart(evt: DragEvent) {
		if (!reorderable || !evt.dataTransfer) return;

		totalDelta = 0;
		dragElement = evt.currentTarget as HTMLElement;

		evt.dataTransfer.effectAllowed = 'move';
		evt.dataTransfer.setData('text/html', dragElement.outerHTML);
	}

	function onDragOver(evt: DragEvent) {
		if (!reorderable || !dragElement) return;

		let target = evt.currentTarget as HTMLElement;
		let draggingUuid = dragElement.dataset.uuid;
		let targetUuid = target.dataset.uuid;

		if (draggingUuid === targetUuid) return;

		if (isBefore(dragElement, target)) {
			onReorder(draggingUuid!, -1);
			totalDelta--;
		} else {
			onReorder(draggingUuid!, 1);
			totalDelta++;
		}

		dragElement = target;
	}

	async function onDragEnd(evt: DragEvent) {
		if (!reorderable || !dragElement) return;

		let uuid = dragElement.dataset.uuid!;

		if ($profileQuery.sortOrder === SortOrder.Descending) {
			// list is reversed
			totalDelta *= -1;
		}

		await invokeCommand('reorder_mod', { uuid, delta: totalDelta });
		await refresh();

		dragElement = null;
		totalDelta = 0;
	}
</script>

<ModList {sortOptions} bind:mods bind:activeMod queryArgs={profileQuery}>
	<div slot="details">
		{#if activeMod && isOutdated(activeMod)}
			<Button.Root
				class="mt-2 flex w-full items-center justify-center gap-2 rounded-lg bg-green-600
						py-2 text-lg font-medium hover:bg-green-500"
				on:click={() => updateActiveMod('latest')}
			>
				<Icon icon="mdi:arrow-up-circle" class="align-middle text-xl" />
				Update to {activeMod?.versions[0].name}
			</Button.Root>
		{/if}
	</div>

	<svelte:fragment slot="dropdown">
		{#if activeMod && activeMod?.versions.length > 1}
			<DropdownMenu.Sub>
				<DropdownMenu.SubTrigger
					class="flex cursor-default items-center truncate rounded-md py-1 pl-3 pr-1 
							text-left text-slate-300 hover:bg-gray-600 hover:text-slate-100"
				>
					<Icon class="mr-1.5 text-lg" icon="mdi:edit" />
					Change version
					<Icon class="ml-4 text-xl" icon="mdi:chevron-right" />
				</DropdownMenu.SubTrigger>
				<DropdownMenu.SubContent
					class="mr-2 flex max-h-96 flex-col gap-0.5 overflow-y-auto rounded-lg
							border border-gray-500 bg-gray-700 p-1 shadow-xl"
					transition={fly}
					transitionConfig={{ duration: 50 }}
				>
					{#each activeMod?.versions ?? [] as version}
						<DropdownMenu.Item
							class="flex flex-shrink-0 cursor-default items-center truncate rounded-md py-1 pl-3 pr-12 
									text-left text-slate-300 hover:bg-gray-600 hover:text-slate-100"
							on:click={() => updateActiveMod({ specific: version.uuid })}
						>
							{version.name}
						</DropdownMenu.Item>
					{/each}
				</DropdownMenu.SubContent>
			</DropdownMenu.Sub>
		{/if}

		<ModDetailsDropdownItem
			label="Uninstall"
			icon="mdi:delete"
			onClick={() =>
				uninstall({
					uuid: activeMod?.uuid ?? '',
					fullName: activeMod?.name ?? ''
				})}
		/>

		<ModDetailsDropdownItem
			icon="mdi:source-branch"
			label="Show dependants"
			onClick={openDependants}
		/>

		<ModDetailsDropdownItem
			label="Open directory"
			icon="mdi:folder"
			onClick={() => invokeCommand('open_plugin_dir', { uuid: activeMod?.uuid })}
		/>
	</svelte:fragment>

	<div slot="banner">
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

		{#if updates.length > $updateBannerThreshold}
			<div
				class="mb-1 mr-3 flex items-center rounded-lg bg-green-700 py-1 pl-3 pr-1 text-green-100"
			>
				<Icon icon="mdi:arrow-up-circle" class="mr-2 text-xl" />
				There {updates.length === 1 ? 'is' : 'are'}
				<strong class="mx-1">{updates.length}</strong>
				{updates.length === 1 ? ' update' : ' updates'} available.
				<Button.Root
					class="ml-1 font-semibold text-white hover:text-green-200 hover:underline"
					on:click={() => {
						updateAllOpen = true;
					}}
				>
					Update all?
				</Button.Root>

				<Button.Root
					class="ml-auto rounded-md p-1 text-xl hover:bg-green-600"
					on:click={() => ($updateBannerThreshold = updates.length)}
				>
					<Icon icon="mdi:close" />
				</Button.Root>
			</div>
		{/if}
	</div>

	<svelte:fragment slot="item" let:mod let:isSelected>
		<ProfileModListItem
			{mod}
			{isSelected}
			{reorderable}
			on:dragstart={onDragStart}
			on:dragend={onDragEnd}
			on:dragover={onDragOver}
			on:toggle={({ detail: { mod, newState } }) => toggleMod(mod, newState)}
		/>
	</svelte:fragment>
</ModList>

<ConfirmPopup title="Confirm update" bind:open={updateAllOpen}>
	Select which mods to update:

	<Checklist
		title="Update all"
		set={(update, _, value) => {
			includeUpdates.set(update, value);
			includeUpdates = includeUpdates; // force reactivity
		}}
		get={(update, _) => includeUpdates.get(update) ?? true}
		items={updates}
		let:item
		class="mt-1 overflow-y-auto"
	>
		<ModCard fullName={item.fullName} showVersion={false} />

		<span class="text-light ml-auto text-slate-400">{item.old}</span>
		<Icon icon="mdi:arrow-right" class="mx-1.5 text-lg text-slate-400" />
		<span class="text-lg font-semibold text-green-400">{item.new}</span>

		<Tooltip text="Ignore this update in the 'Update all' list." side="left" sideOffset={-2}>
			<Button.Root
				class="ml-2 rounded p-1.5 text-slate-400 hover:bg-gray-700 hover:text-slate-200"
				on:click={() => {
					item.ignore = true;
					allUpdates = allUpdates; // force reactivity

					includeUpdates.delete(item);
					includeUpdates = includeUpdates; // force reactivity
					invokeCommand('ignore_update', { versionUuid: item.versionUuid });
				}}><Icon icon="mdi:notifications-off" /></Button.Root
			>
		</Tooltip>
	</Checklist>

	<svelte:fragment slot="buttons">
		<BigButton
			color="green"
			fontWeight="semibold"
			on:click={() => {
				let uuids = updates
					.filter((update) => includeUpdates.get(update) ?? true)
					.map((update) => update.packageUuid);

				invokeCommand('update_mods', { uuids, respectIgnored: true }).then(refresh);
				updateAllOpen = false;
			}}
		>
			Update mods
		</BigButton>
	</svelte:fragment>
</ConfirmPopup>

<Popup title="Dependants of {activeMod?.name}" bind:open={dependantsOpen}>
	<div class="mt-4 text-center text-slate-300">
		{#if dependants}
			{#if dependants.length === 0}
				No dependants found
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
		activeMod = undefined;
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
