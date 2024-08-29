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
	import { activeProfile, profileQuery } from '$lib/stores';
	import { isOutdated } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Button, DropdownMenu, Switch } from 'bits-ui';
	import { fly } from 'svelte/transition';
	import Popup from '$lib/components/Popup.svelte';
	import { onMount } from 'svelte';
	import ModCardList from '$lib/modlist/ModCardList.svelte';
	import ModCard from '$lib/modlist/ModCard.svelte';
	import Checklist from '$lib/components/Checklist.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { t, T } from '$i18n';

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

	async function toggleMod(enable: boolean, mod: Mod) {
		let response = await invokeCommand<ModActionResponse>('toggle_mod', {
			uuid: mod.uuid
		});

		if (response.type == 'done') {
			refresh();
			return;
		}

		if (enable) {
			enableDependencies.openFor(mod, response.content);
		} else {
			disableDependants.openFor(mod, response.content);
		}
	}

	async function uninstall(mod: Dependant) {
		let response = await invokeCommand<ModActionResponse>('remove_mod', { uuid: mod.uuid });

		if (response.type == 'done') {
			refresh();
			activeMod = undefined;
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

	async function finishReorder(uuid: string, delta: number) {
		if ($profileQuery.sortOrder === SortOrder.Descending) {
			// list is reversed
			delta *= -1;
		}

		await invokeCommand('reorder_mod', { uuid, delta });
		await refresh();
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
</script>

<ModList
	{sortOptions}
	{reorderable}
	bind:mods
	bind:activeMod
	on:reorder={({ detail: { uuid, delta } }) => onReorder(uuid, delta)}
	on:finishReorder={({ detail: { uuid, totalDelta } }) => finishReorder(uuid, totalDelta)}
	queryArgs={profileQuery}
>
	<div slot="details">
		{#if activeMod && isOutdated(activeMod)}
			<Button.Root
				class="flex items-center justify-center w-full gap-2 py-2 rounded-lg mt-2
						bg-green-600 hover:bg-green-500 font-medium text-lg"
				on:click={() => updateActiveMod('latest')}
			>
				<Icon icon="mdi:arrow-up-circle" class="text-xl align-middle" />
				{T(t["Update to version"], {"version": activeMod?.versions[0].name})}
			</Button.Root>
		{/if}
	</div>

	<svelte:fragment slot="dropdown">
		{#if activeMod && activeMod?.versions.length > 1}
			<DropdownMenu.Sub>
				<DropdownMenu.SubTrigger
					class="flex items-center pl-3 pr-1 py-1 truncate text-slate-300 hover:text-slate-100 
							text-left rounded-md hover:bg-gray-600 cursor-default"
				>
					<Icon class="text-lg mr-1.5" icon="mdi:edit" />
					{t["Change version"]}
					<Icon class="text-xl ml-4" icon="mdi:chevron-right" />
				</DropdownMenu.SubTrigger>
				<DropdownMenu.SubContent
					class="flex flex-col max-h-96 overflow-y-auto bg-gray-700 gap-0.5 mr-2
							shadow-xl p-1 rounded-lg border border-gray-500"
					transition={fly}
					transitionConfig={{ duration: 50 }}
				>
					{#each activeMod?.versions ?? [] as version}
						<DropdownMenu.Item
							class="flex flex-shrink-0 items-center pl-3 pr-12 py-1 truncate text-slate-300 hover:text-slate-100 
									text-left rounded-md hover:bg-gray-600 cursor-default"
							on:click={() => updateActiveMod({ specific: version.uuid })}
						>
							{version.name}
						</DropdownMenu.Item>
					{/each}
				</DropdownMenu.SubContent>
			</DropdownMenu.Sub>
		{/if}

		<ModDetailsDropdownItem
			label="{t["Open directory"]}"
			icon="mdi:folder"
			onClick={() => invokeCommand('open_plugin_dir', { uuid: activeMod?.uuid })}
		/>

		{#if activeMod?.type === 'remote'}
			<ModDetailsDropdownItem
				icon="mdi:source-branch"
				label="{t["Show dependants"]}"
				onClick={openDependants}
			/>
		{/if}

		<ModDetailsDropdownItem
			label="{t["Uninstall"]}"
			icon="mdi:delete"
			onClick={() =>
				uninstall({
					uuid: activeMod?.uuid ?? '',
					name: activeMod?.name ?? ''
				})}
		/>
	</svelte:fragment>

	<div slot="banner">
		{#if unknownMods.length > 0}
			<div class="flex items-center text-red-100 bg-red-600 mr-3 mb-1 pl-3 pr-1 py-1.5 rounded-lg">
				<Icon icon="mdi:alert-circle" class="text-xl mr-2" />
				{unknownMods.length === 1 ? t["Following mod not found"] : t["Following mods not found"]} 
				{unknownMods
					.map((mod) => mod.name)
					.join(', ')}.
				<Button.Root
					class="hover:underline hover:text-red-100 text-white font-semibold ml-1"
					on:click={() => {
						unknownMods.forEach(uninstall);
					}}
				>
					{t["Uninstall them"]}
				</Button.Root>
			</div>
		{/if}

		{#if updates.length > $updateBannerThreshold}
			<div
				class="flex items-center text-green-100 bg-green-700 mr-3 mb-1 pl-3 pr-1 py-1 rounded-lg"
			>
				<Icon icon="mdi:arrow-up-circle" class="text-xl mr-2" />
				{@html updates.length === 1 
				? T(t["A mod update available"], {"length": updates.length.toString()})
				: T(t["Many mod update available"], {"length": updates.length.toString()})}
				<Button.Root
					class="hover:underline hover:text-green-200 text-white font-semibold ml-1"
					on:click={() => {
						updateAllOpen = true;
					}}
				>
					{t["Update all question"]}
				</Button.Root>

				<Button.Root
					class="ml-auto rounded-md text-xl hover:bg-green-600 p-1"
					on:click={() => ($updateBannerThreshold = updates.length)}
				>
					<Icon icon="mdi:close" />
				</Button.Root>
			</div>
		{/if}
	</div>

	<div slot="item" let:mod>
		<div class="flex items-center mt-2.5 ml-1">
			{#if reorderable}
				<Icon
					icon="material-symbols:drag-indicator"
					class="text-slate-400 text-2xl cursor-move mr-3"
				/>
			{/if}

			<Switch.Root
				checked={mod.enabled ?? true}
				onCheckedChange={(checked) => toggleMod(checked, mod)}
				on:click={(evt) => evt.stopPropagation()}
				class="flex px-1 py-1 mr-1 rounded-full w-12 h-6 group
						bg-slate-600 hover:bg-slate-500
						data-[state=checked]:bg-green-700 data-[state=checked]:hover:bg-green-600"
			>
				<Switch.Thumb
					class="pointer-events-none h-full w-4 rounded-full transition-transform ease-out duration-75
							bg-slate-300 hover:bg-slate-200
							data-[state=checked]:translate-x-6 data-[state=checked]:bg-green-200 data-[state=checked]:group-hover:bg-green-100"
				/>
			</Switch.Root>
		</div>
	</div>
</ModList>

<ConfirmPopup title="{t["Confirm update"]}" bind:open={updateAllOpen}>
	{t["Confirm update description"]}

	<Checklist
		title="{t["Update all"]}"
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

		<span class="text-slate-400 text-light ml-auto">{item.old}</span>
		<Icon icon="mdi:arrow-right" class="text-slate-400 text-lg mx-1.5" />
		<span class="text-green-400 font-semibold text-lg">{item.new}</span>

		<Tooltip text="{t["Ignore in update all"]}" side="left" sideOffset={-2}>
			<Button.Root
				class="ml-2 p-1.5 text-slate-400 hover:text-slate-200 hover:bg-gray-700 rounded"
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
			{t["Update mods"]}
		</BigButton>
	</svelte:fragment>
</ConfirmPopup>

<Popup title="{T(t["Dependants of"], {"name": activeMod?.name})}" bind:open={dependantsOpen}>
	<div class="text-slate-300 text-center mt-4">
		{#if dependants}
			{#if dependants.length === 0}
				{t["No dependants found"]}
			{:else}
				<ModCardList names={dependants} />
			{/if}
		{:else}
			{t["Loading"]}
		{/if}
	</div>
</Popup>

<DependantsPopup
	bind:this={removeDependants}
	title="{t["Confirm removal"]}"
	verb="{t["Remove"]}"
	description="{t["Confirm removal description"]}"
	commandName="remove_mod"
	onExecute={() => {
		refresh();
		activeMod = undefined;
	}}
	onCancel={refresh}
/>

<DependantsPopup
	bind:this={disableDependants}
	title="{t["Confirm disabling"]}"
	verb="{t["Disable"]}"
	description="{t["Confirm disabling description"]}"
	commandName="toggle_mod"
	onExecute={refresh}
	onCancel={refresh}
/>

<DependantsPopup
	bind:this={enableDependencies}
	title="{t["Confirm enabling"]}"
	verb="{t["Enable"]}"
	description="{t["Confirm enabling description"]}"
	commandName="toggle_mod"
	onExecute={refresh}
	onCancel={refresh}
	isPositive={true}
/>
