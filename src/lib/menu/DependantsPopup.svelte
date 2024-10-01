<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { Dependant, Mod } from '$lib/models';
	import ModCardList from '$lib/modlist/ModCardList.svelte';

	export let title: string;
	export let verb: string;
	export let description: string;
	export let commandName: string;
	export let positive: boolean = false;
	export let onExecute: () => void;
	export let onCancel: () => void;

	let name: string;
	let uuid: string;
	let open: boolean;
	let dependants: Dependant[];

	export function openFor(_mod: Dependant | Mod, _dependants: Dependant[]) {
		if ('fullName' in _mod) {
			name = _mod.fullName;
		} else {
			name = _mod.name;
		}

		uuid = _mod.uuid;
		dependants = _dependants;
		open = true;
	}

	async function executeAll() {
		await execute(dependants.map(({ uuid }) => uuid).concat(uuid));
	}

	async function executeOne() {
		await execute([uuid]);
	}

	async function execute(uuids: string[]) {
		await invokeCommand('force_' + commandName + 's', { uuids });
		open = false;
		dependants = [];
		onExecute();
	}
</script>

<ConfirmPopup {title} {onCancel} bind:open>
	{description.replaceAll('%s', name)}

	<ModCardList
		class="my-2 max-h-[50vh] overflow-y-auto"
		names={dependants.map(({ fullName }) => fullName)}
	/>

	<svelte:fragment slot="buttons">
		<BigButton on:click={executeOne} color="gray">
			{verb}
			{name} only
		</BigButton>
		<BigButton on:click={executeAll} color={positive ? 'green' : 'red'} fontWeight="semibold">
			{verb} all
		</BigButton>
	</svelte:fragment>
</ConfirmPopup>
