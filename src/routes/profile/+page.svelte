<script lang="ts">
	import Popup from '$lib/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { Dependant, Mod, QueryModsArgs, RemoveModResponse } from '$lib/models';
	import ModList from '$lib/modlist/ModList.svelte';
	import { currentProfile } from '$lib/profile';
	import { Button, Dialog } from 'bits-ui';

	let mods: Mod[];
	let activeMod: Mod | undefined;
	let queryArgs: QueryModsArgs;

	let dependants: Dependant[] = [];
	let removeDependantsPopupOpen = false;

	$: {
		queryArgs;
		$currentProfile;
		refresh();
	}

	function refresh() {
		if (queryArgs) {
			invokeCommand<Mod[]>('query_mods_in_profile', { args: queryArgs })
				.then((result) => mods = result)
		}
	}

	async function removeMod() {
		if (!activeMod) return;

		let response = await invokeCommand<RemoveModResponse>('remove_mod', { 
			packageUuid: activeMod.package.uuid4
		});

		if (response.type == 'removed') {
			activeMod = undefined;
			refresh();
			return;
		}

		dependants = response.data;
		removeDependantsPopupOpen = true;
	}

	function forceRemoveDependants() {
		let packageUuids = dependants.map(d => d.uuid).concat(activeMod!.package.uuid4);

		invokeCommand('force_remove_mods', { packageUuids }).then(() => {
			activeMod = undefined;
			dependants = [];
			removeDependantsPopupOpen = false;
			refresh();
		})
	}
</script>

<ModList bind:mods bind:queryArgs bind:activeMod extraDropdownOptions={
	[
		{
			icon: 'mdi:delete',
			label: 'Uninstall',
			onClick: removeMod
		}
	]
}/>

{#if activeMod}
	<Popup 
		title="Confirm removal"
		bind:open={removeDependantsPopupOpen}
	>
		<Dialog.Description class="text-slate-300">
			The following mods depend on {activeMod.package.name} and <strong>will be also be removed if you proceed:</strong>
			<ul class="mt-1">
				{#each dependants as dependant}
					<li>- {dependant.name}</li>
				{/each}
			</ul>
		</Dialog.Description>

		<div class="flex w-full justify-end mt-3 mr-0.5 gap-2">
			<Dialog.Close>
				<Button.Root
					class="rounded-xl px-4 py-2 text-gray-200 bg-gray-700 hover:bg-gray-600"
					on:click={forceRemoveDependants}
				>
					Proceed
				</Button.Root>
			</Dialog.Close>
			<Dialog.Close>
				<Button.Root
					class="rounded-xl px-4 py-2 text-slate-100 bg-red-600 hover:bg-red-500"
					on:click={() => removeDependantsPopupOpen = false}
				>
					Cancel
				</Button.Root>
			</Dialog.Close>
		</div>
	</Popup>
{/if}
