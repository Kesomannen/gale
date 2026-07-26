<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { Dependant } from '$lib/types';
	import InfoBox from '../ui/InfoBox.svelte';
	import Button from '../ui/Button.svelte';
	import Dialog from '../ui/Dialog.svelte';
	import ModCardList from '../ui/ModCardList.svelte';

	type Props = {
		mods: Dependant[];
		uninstall: (mod: Dependant) => void;
	};

	let { mods, uninstall }: Props = $props();

	let dialogOpen = $state(false);
</script>

<InfoBox type="warning">
	<div class="flex items-center justify-between">
		<span>{m.unknownModsBanner_content()}</span>
		<Button color="primary" icon="mdi:info" onclick={() => (dialogOpen = true)}
			>{m.unknownModsBanner_details_content()}</Button
		>
	</div>
</InfoBox>

<Dialog bind:open={dialogOpen} title="Unknown mods">
	<div class="text-primary-300 mb-3">
		{m.unknownModsBanner_dialog_content_1()}
		<br />
		{m.unknownModsBanner_dialog_content_2()}
	</div>

	<ModCardList {mods} />

	<div class="mt-2 flex justify-end gap-2">
		<Button color="primary" onclick={() => (dialogOpen = false)}
			>{m.unknownModsBanner_ignore_content()}</Button
		>
		<Button
			icon="mdi:trash"
			color="primary"
			onclick={() => {
				mods.forEach(uninstall);
				dialogOpen = false;
			}}
		>
			{m.unknownModsBanner_uninstall_content()}
		</Button>
	</div>
</Dialog>
