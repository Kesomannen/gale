<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
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

		<Dialog.Close class="flex w-full justify-end mt-3 mr-0.5 gap-2">
			<BigButton onClick={onCancel} color="gray">Cancel</BigButton>
			<BigButton onClick={executeOne} color="red" outline={true}>
				{verb}
				{mod.name} only
			</BigButton>
			<BigButton onClick={executeAll} color={isPositive ? 'blue' : 'red'} fontWeight="semibold">
				{verb} all
			</BigButton>
		</Dialog.Close>
	{/if}
</Popup>
