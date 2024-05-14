<script lang="ts">
	import Popup from '$lib/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import DependantsPopup from '$lib/menu/DependantsPopup.svelte';
	import type { Dependant, Mod, QueryModsArgs, RemoveModResponse } from '$lib/models';
	import ModList from '$lib/modlist/ModList.svelte';
	import { currentGame, currentProfile } from '$lib/profile';
	import { isOutdated } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Button, Dialog, Switch, Tooltip } from 'bits-ui';

	let mods: Mod[];
	let activeMod: Mod | undefined;
	let queryArgs: QueryModsArgs;

	let removeDependants: DependantsPopup;
	let disableDependants: DependantsPopup;

	$: outdatedMods = mods?.filter((mod) => isOutdated(mod));

	$: {
		queryArgs;
		$currentProfile;
		$currentGame;
		refresh();
	}

	function refresh() {
		if (queryArgs) {
			invokeCommand<Mod[]>('query_mods_in_profile', { args: queryArgs }).then(
				(result) => (mods = result)
			);
		}
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
			onClick: () => removeDependants.tryExecute(activeMod)
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
				Update
			</Button.Root>
		{/if}
	</div>
	<div slot="header">
		{#if outdatedMods?.length > 0}
			<div class="text-slate-200 bg-blue-600 ml-2 mr-6 mb-2 px-4 py-2 rounded-lg">
				<Icon icon="mdi:arrow-up-circle" class="text-xl mr-1 mb-0.5 inline" />
				There {outdatedMods.length === 1 ? 'is' : 'are'} <strong>{outdatedMods.length}</strong>
				updates available.
				<Button.Root
					class="underline"
					on:click={() => {
						invokeCommand('update_all').then(refresh);
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
			onCheckedChange={evt => {
				disableDependants.tryExecute(mod);
			}}
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

<DependantsPopup
	bind:this={removeDependants}
	verb="Remove"
	commandName="remove_mod"
	onExecute={() => {
		refresh();
		activeMod = undefined;
	}}
/>

<DependantsPopup
	bind:this={disableDependants}
	verb="Disable"
	commandName="toggle_mod"
	onExecute={refresh}
/>