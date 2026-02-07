<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { invoke } from '$lib/invoke';
	import ModCardList from '$lib/components/ui/ModCardList.svelte';
	import type { Dependant, Mod } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		title: string;
		verb: string;
		description: string;
		commandName: string;
		positive?: boolean;
		onExecute?: () => void;
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
		await invoke('force_' + commandName + 's', { uuids });
		open = false;
		dependants = [];
		onExecute?.();
	}
</script>

<ConfirmDialog {title} {onCancel} bind:open>
	{description.replaceAll('%s', name)}

	<ModCardList
		class="my-2 max-h-[50vh] overflow-y-auto"
		names={dependants.map(({ fullName }) => fullName)}
		showVersion={false}
	/>

	{#snippet buttons()}
		<Button onclick={executeOne} color="primary" class="truncate">
			{m.dependantsDialog_button_executeOne({ verb, name })}
		</Button>

		<Button onclick={executeAll} color={positive ? 'accent' : 'red'}>
			{m.dependantsDialog_button_executeAll({ verb })}
		</Button>
	{/snippet}
</ConfirmDialog>
