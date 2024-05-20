<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import DependantsPopup from '$lib/menu/DependantsPopup.svelte';
	import type {
		Dependant,
		Mod,
		ModActionResponse,
		QueryModsArgs,
		ProfileQuery,
		AvailableUpdate
	} from '$lib/models';
	import ModList from '$lib/modlist/ModList.svelte';
	import { currentGame, currentProfile } from '$lib/profile';
	import { isOutdated } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { tauri } from '@tauri-apps/api';
	import { Button, Dialog, Switch, Tooltip } from 'bits-ui';
	import { onMount } from 'svelte';

	let mods: Mod[];
	let updates: AvailableUpdate[] = [];
	let activeMod: Mod | undefined;
	let queryArgs: QueryModsArgs;

	let removeDependants: DependantsPopup;
	let disableDependants: DependantsPopup;
	let enableDependencies: DependantsPopup;

	let updateAllOpen = false;

	$: {
		queryArgs;
		$currentProfile;
		$currentGame;
		refresh();
	}

	function refresh() {
		if (queryArgs) {
			invokeCommand<ProfileQuery>('query_mods_in_profile', { args: queryArgs }).then((result) => {
				mods = result.mods;
				updates = result.updates;
			});
		}
	}

	async function toggleMod(new_value: boolean, mod: Mod) {
		let response = await invokeCommand<ModActionResponse>('toggle_mod', {
			uuid: mod.uuid
		});

		if (response.type == 'done') {
			refresh();
			return;
		}

		if (new_value) {
			enableDependencies.openFor(mod, response.content);
		} else {
			disableDependants.openFor(mod, response.content);
		}
	}

	async function removeMod() {
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
</script>

<ModList
	bind:mods
	bind:queryArgs
	bind:activeMod
	extraDropdownOptions={[
		{
			icon: 'mdi:delete',
			label: 'Uninstall',
			onClick: removeMod
		}
	]}
>
	<div slot="details">
		{#if activeMod && isOutdated(activeMod)}
			<Button.Root
				class="flex items-center justify-center w-full gap-2 py-2 rounded-lg mt-2
						bg-blue-600 hover:bg-blue-500 font-medium text-lg"
				on:click={() => {
					invokeCommand('update_mod', { uuid: activeMod?.uuid }).then(refresh);
				}}
			>
				<Icon icon="mdi:arrow-up-circle" class="text-xl align-middle" />
				Update to {activeMod?.versions[0].name}
			</Button.Root>
		{/if}
	</div>
	<div slot="header">
		{#if updates.length > 0}
			<div class="text-blue-100 bg-blue-600 ml-2 mr-6 mb-2 px-4 py-2 rounded-lg">
				<Icon icon="mdi:arrow-up-circle" class="text-xl mr-1 mb-0.5 inline" />
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
			</div>
		{/if}
	</div>
	<div slot="item" let:mod>
		<Switch.Root
			checked={mod.enabled ?? true}
			onCheckedChange={(checked) => toggleMod(checked, mod)}
			on:click={(evt) => evt.stopPropagation()}
			class="peer flex items-center px-1 py-1 rounded-full w-12 h-6 ml-2 group
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

<Popup title="Confirm update" bind:open={updateAllOpen}>
	<Dialog.Description class="text-slate-400">
		The following mods will be updated:
	</Dialog.Description>

	<ul class="mt-2">
		{#each updates as update}
			<li>
				<span class="text-slate-300">{update.name}</span>
				<span class="text-slate-400 text-light">{update.old} > </span>
				<span class="text-slate-100 font-medium">{update.new}</span>
			</li>
		{/each}
	</ul>

	<div class="flex w-full justify-end mt-3 mr-0.5 gap-2">
		<Dialog.Close>
			<Button.Root class="rounded-xl px-4 py-2 text-slate-100 bg-gray-700 hover:bg-gray-600">
				Cancel
			</Button.Root>
		</Dialog.Close>
		<Dialog.Close>
			<Button.Root
				class="rounded-xl px-4 py-2 font-medium text-white bg-blue-600 hover:bg-blue-500"
				on:click={() => {
					invokeCommand('update_all').then(refresh);
					updateAllOpen = false;
				}}
			>
				Update all
			</Button.Root>
		</Dialog.Close>
	</div>
</Popup>

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
