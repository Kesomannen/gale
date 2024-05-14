<script lang="ts">
	import Popup from '$lib/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { Dependant, Mod, RemoveModResponse as MoActionResponse } from '$lib/models';
	import { Button, Dialog } from 'bits-ui';

	export let verb: string;
	export let commandName: string;
	export let onExecute: () => void;
  
  let mod: Mod | undefined;
	let open: boolean;
	let dependants: Dependant[] = []
  
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

	export async function tryExecute(target_mod: Mod | undefined) {
    mod = target_mod;
		if (!mod) return;

		let response = await invokeCommand<MoActionResponse>(commandName, {
			uuid: mod.uuid
		});

		if (response.type == 'done') {
			mod = undefined;
			dependants = [];
      onExecute();
			return;
		}

		dependants = response.content;
		open = true;
	}
</script>

<Popup title="Confirm action" bind:open onClose={onExecute}>
	{#if mod}
		<Dialog.Description class="text-slate-300">
			The following mods depend on {mod.name} and
			<strong>will likely not work if {mod.name} is {verb.toLowerCase()}d!</strong>
			<ul class="mt-1">
				{#each dependants as dependant}
					<li>- {dependant.name}</li>
				{/each}
			</ul>
		</Dialog.Description>

		<div class="flex w-full justify-end mt-3 mr-0.5 gap-2">
			<Dialog.Close>
				<Button.Root class="rounded-xl px-4 py-2 text-slate-100 bg-gray-700 hover:bg-gray-600">
					Cancel
				</Button.Root>
			</Dialog.Close>
			<Dialog.Close>
				<Button.Root
					class="rounded-xl px-4 py-2 text-red-400 hover:text-red-300 border border-red-500 hover:border-red-400"
					on:click={executeOne}
				>
					{verb} {mod.name} only
				</Button.Root>
			</Dialog.Close>
			<Dialog.Close>
				<Button.Root
					class="rounded-xl px-4 py-2 text-white bg-red-600 hover:bg-red-500"
					on:click={executeAll}
				>
					{verb} all
				</Button.Root>
			</Dialog.Close>
		</div>
	{/if}
</Popup>
