<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { Dependant, Mod, ModActionResponse } from '$lib/models';
	import { Button, Dialog } from 'bits-ui';

	export let title: string;
	export let verb: string;
	export let description: string;
	export let commandName: string;
	export let isPositive: boolean = false;
	export let onExecute: () => void;
	export let onCancel: () => void;
  
  let mod: Mod | undefined;
	let open: boolean;
	let dependants: Dependant[];
  
	export function openFor(_mod: Mod, _dependants: Dependant[]) {
		mod = _mod;
		dependants = _dependants;
		open = true;
	}

  async function executeAll() {
    if (!mod) return;
    await execute(dependants.map((d) => d.uuid).concat(mod.uuid));
  }

  async function executeOne() {
    if (!mod) return;
    await execute([mod.uuid]);
  }

	async function execute(uuids: string[]) {
		await invokeCommand('force_' + commandName + 's', { uuids });
		open = false;
		dependants = [];
		onExecute();
	}
</script>

<Popup {title} bind:open onClose={onExecute}>
	{#if mod}
		<Dialog.Description class="text-slate-300">
			{description.replaceAll('%s', mod.name)}
			<ul class="mt-1">
				{#each dependants as dependant}
					<li>- {dependant.name}</li>
				{/each}
			</ul>
		</Dialog.Description>

		<div class="flex w-full justify-end mt-3 mr-0.5 gap-2">
			<Dialog.Close>
				<Button.Root 
					class="rounded-xl px-4 py-2 text-slate-100 bg-gray-700 hover:bg-gray-600"
					on:click={onCancel}
				>
					Cancel
				</Button.Root>
			</Dialog.Close>
			<Dialog.Close>
				<Button.Root
					class="rounded-xl px-4 py-2 font-semibold text-red-400 hover:text-red-300 border border-red-500 hover:border-red-400"
					on:click={executeOne}
				>
					{verb} {mod.name} only
				</Button.Root>
			</Dialog.Close>
			<Dialog.Close>
				<Button.Root
					class="rounded-xl px-4 py-2 font-medium text-white {isPositive ? 'bg-blue-600 hover:bg-blue-500' : 'bg-red-600 hover:bg-red-500'}"
					on:click={executeAll}
				>
					{verb} all
				</Button.Root>
			</Dialog.Close>
		</div>
	{/if}
</Popup>
