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
	} from '$lib/models';
	import ModDetailsDropdownItem from '$lib/modlist/ModDetailsDropdownItem.svelte';
	import ModList from '$lib/modlist/ModList.svelte';
	import { currentProfile, profileQuery } from '$lib/stores';
	import { isOutdated } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Button, DropdownMenu, Switch } from 'bits-ui';
	import { fly } from 'svelte/transition';

	const sortOptions = [
		SortBy.Custom,
		SortBy.InstallDate,
		SortBy.LastUpdated,
		SortBy.Newest,
		SortBy.Rating,
		SortBy.Downloads,
		SortBy.Name,
		SortBy.Author
	];

	let mods: Mod[] = [];
	let updates: AvailableUpdate[] = [];
	let activeMod: Mod | undefined;

	let removeDependants: DependantsPopup;
	let disableDependants: DependantsPopup;
	let enableDependencies: DependantsPopup;

	let updateAllOpen = false;

	$: {
		$currentProfile;
		$profileQuery;
		refresh();
	}

	// really ugly hack because reactive statements run once immediately no matter what :/
	let isFirst = true;

	$: {
		$currentProfile;
		resetBannerThreshold();
	}

	function resetBannerThreshold() {
		if (isFirst) {
			isFirst = false;
			return;
		}

		$updateBannerThreshold = 0;
	}

	function refresh() {
		invokeCommand<ProfileQuery>('query_profile', { args: $profileQuery }).then((result) => {
			mods = result.mods;
			updates = result.updates;
		});
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

	async function uninstall() {
		if (!activeMod) return;

		let response = await invokeCommand<ModActionResponse>('remove_mod', {
			uuid: activeMod.uuid
		});

		if (response.type == 'done') {
			refresh();
			activeMod = undefined;
			return;
		}

		removeDependants.openFor(activeMod, response.content);
	}

	async function changeVersion(uuid: string) {
		if (!activeMod) return;

		await invokeCommand('update_mod', { uuid: activeMod.uuid, version: { specific: uuid } });
		refresh();
	}

	async function onReorder(uuid: string, delta: number, name: string) {
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
</script>

<ModList
	bind:mods
	bind:activeMod
	on:reorder={({ detail: { uuid, delta, name } }) => onReorder(uuid, delta, name)}
	{sortOptions}
	reorderable={$profileQuery.sortBy === SortBy.Custom}
	queryArgs={profileQuery}
>
	<div slot="details">
		{#if activeMod && isOutdated(activeMod)}
			<Button.Root
				class="flex items-center justify-center w-full gap-2 py-2 rounded-lg mt-2
						bg-blue-600 hover:bg-blue-500 font-medium text-lg"
				on:click={() => {
					invokeCommand('update_mod', { uuid: activeMod?.uuid, version: 'latest' }).then(refresh);
				}}
			>
				<Icon icon="mdi:arrow-up-circle" class="text-xl align-middle" />
				Update to {activeMod?.versions[0].name}
			</Button.Root>
		{/if}
	</div>

	<svelte:fragment slot="dropdown">
		<DropdownMenu.Sub>
			<DropdownMenu.SubTrigger
				class="flex items-center pl-3 pr-1 py-1 truncate text-slate-300 hover:text-slate-100 
							text-left rounded-md hover:bg-gray-600 cursor-default"
			>
				<Icon class="text-lg mr-1.5" icon="mdi:edit" />
				Change version
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
						on:click={() => changeVersion(version.uuid)}
					>
						{version.name}
					</DropdownMenu.Item>
				{/each}
			</DropdownMenu.SubContent>
		</DropdownMenu.Sub>

		<ModDetailsDropdownItem
			label="Open directory"
			icon="mdi:folder"
			onClick={() => invokeCommand('open_plugin_dir', { uuid: activeMod?.uuid })}
		/>

		<ModDetailsDropdownItem label="Uninstall" icon="mdi:delete" onClick={uninstall} />
	</svelte:fragment>

	<div slot="banner">
		{#if updates.length > $updateBannerThreshold}
			<div class="flex items-center text-blue-100 bg-blue-600 mr-3 mb-2 pl-4 pr-2 py-1.5 rounded-lg gap-1">
				<Icon icon="mdi:arrow-up-circle" class="text-xl" />
				There {updates.length === 1 ? 'is' : 'are'} <strong>{updates.length}</strong>
				{updates.length === 1 ? ' update' : ' updates'} available.
				<Button.Root
					class="hover:underline hover:text-blue-100 text-slate-100 font-semibold"
					on:click={() => {
						updateAllOpen = true;
					}}
				>
					Update all?
				</Button.Root>

				<Button.Root 
					class="ml-auto rounded-md text-xl hover:bg-blue-500 p-1"
					on:click={() => $updateBannerThreshold = updates.length}
				>
					<Icon icon="mdi:close" />
				</Button.Root>
			</div>
		{/if}
	</div>

	<div slot="item" let:mod>
		<Switch.Root
			checked={mod.enabled ?? true}
			onCheckedChange={(checked) => toggleMod(checked, mod)}
			on:click={(evt) => evt.stopPropagation()}
			class="flex px-1 py-1 rounded-full w-12 h-6 ml-2 group mt-3
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
</ModList>

<ConfirmPopup title="Confirm update" bind:open={updateAllOpen}>
	The following mods will be updated:

	<ul class="mt-2">
		{#each updates as update}
			<li>
				-
				<span class="text-slate-300">{update.name}</span>
				<span class="text-slate-400 text-light">{update.old} > </span>
				<span class="text-blue-200 font-medium">{update.new}</span>
			</li>
		{/each}
	</ul>

	<svelte:fragment slot="buttons">
		<BigButton
			color="blue"
			fontWeight="semibold"
			on:click={() => {
				invokeCommand('update_all').then(refresh);
				updateAllOpen = false;
			}}
		>
			Update all
		</BigButton>
	</svelte:fragment>
</ConfirmPopup>

<DependantsPopup
	bind:this={removeDependants}
	title="Confirm removal"
	verb="Remove"
	description="The following mods depend on %s and will likely not work if it is removed!"
	commandName="remove_mod"
	onExecute={() => {
		refresh();
		activeMod = undefined;
	}}
	onCancel={refresh}
/>

<DependantsPopup
	bind:this={disableDependants}
	title="Confirm disabling"
	verb="Disable"
	description="The following mods depend on %s and will likely not work if it is disabled!"
	commandName="toggle_mod"
	onExecute={refresh}
	onCancel={refresh}
/>

<DependantsPopup
	bind:this={enableDependencies}
	title="Confirm enabling"
	verb="Enable"
	description="%s depends on the following disabled mods, and will likely not work if any of them are disabled!"
	commandName="toggle_mod"
	onExecute={refresh}
	onCancel={refresh}
	isPositive={true}
/>
