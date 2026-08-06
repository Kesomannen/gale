<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Checkbox from '../ui/Checkbox.svelte';
	import ConfirmDialog from '../ui/ConfirmDialog.svelte';
	import * as api from '$lib/api';
	import Button from '../ui/Button.svelte';
	import Label from '../ui/Label.svelte';

	type Props = {
		open: boolean;
		onConfirm: () => void;
	};

	let { open = $bindable(false), onConfirm }: Props = $props();

	let dontShowAgain = $state(false);

	async function doInstall() {
		if (dontShowAgain) {
			let prefs = await api.prefs.get();
			prefs.backendSkipConfirm = true;
			await api.prefs.set(prefs);
		}

		open = false;
		onConfirm();
	}
</script>

<ConfirmDialog title="Download from other source" bind:open>
	{m.foreignDownloadDialog_content()}

	<div class="my-5 flex items-center">
		<Checkbox id="neverwarninstall" bind:checked={dontShowAgain} />

		<label class="ml-3" for="neverwarninstall"> {m.foreignDownloadDialog_dontShowAgain()} </label>
	</div>

	{#snippet buttons()}
		<Button color="accent" icon="mdi:download" onclick={doInstall}
			>{m.foreignDownloadDialog_continue()}</Button
		>
	{/snippet}
</ConfirmDialog>
