<script lang="ts">
	import BigButton from '$lib/components/Button.svelte';
	import ConfirmPopup from '$lib/components/ConfirmPopup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { Dependant, Mod } from '$lib/models';
	import ModCardList from '$lib/modlist/ModCardList.svelte';

	type Props = {
		title: string;
		verb: string;
		description: string;
		commandName: string;
		positive?: boolean;
		onExecute: () => void;
		onCancel: () => void;
	};

	let {
		title,
		verb,
		description,
		commandName,
		positive = false,
		onExecute,
		onCancel
	}: Props = $props();

	let name: string = $state('');
	let uuid: string;
	let open: boolean = $state(false);
	let dependants: Dependant[] = $state([]);

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
		showVersion={false}
	/>

	{#snippet buttons()}
		<BigButton onclick={executeOne} color="primary" class="truncate">
			{verb}
			{name} only
		</BigButton>
		<BigButton onclick={executeAll} color={positive ? 'accent' : 'red'}>
			{verb} all
		</BigButton>
	{/snippet}
</ConfirmPopup>
