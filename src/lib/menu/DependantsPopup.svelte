<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { Dependant, Mod } from '$lib/models';

	import { get } from 'svelte/store';
	import { T, t } from '$i18n';

	export let title: string;
	export let verb: string;
	export let description: string;
	export let commandName: string;
	export let isPositive: boolean = false;
	export let onExecute: () => void;
	export let onCancel: () => void;

	let mod: Dependant;
	let open: boolean;
	let dependants: Dependant[];

	export function openFor(_mod: Dependant, _dependants: Dependant[]) {
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

<ConfirmPopup
	{title}
	{onCancel}
	bind:open
>
	{description.replaceAll('%s', mod?.name ?? get(t)["Unknown"])}

	<ul class="mt-1">
		{#each dependants as dependant}
			<li>- {dependant.name}</li>
		{/each}
	</ul>
	
	<svelte:fragment slot="buttons">
		<BigButton on:click={executeOne} color="red" outline={true}>
			{T(get(t)['Dependants action only'], {"verb": verb, "name": mod?.name})}
		</BigButton>
		<BigButton on:click={executeAll} color={isPositive ? 'green' : 'red'} fontWeight="semibold">
			{T(get(t)['Dependants action all'], {"verb": verb})}
		</BigButton>
	</svelte:fragment>
</ConfirmPopup>
