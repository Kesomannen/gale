<script lang="ts">
	import DedicatedServerSettingsDialog from '$lib/components/dialogs/DedicatedServerSettingsDialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Info from '$lib/components/ui/Info.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import { m } from '$lib/paraglide/messages';

	let dialogOpen = $state(false);
</script>

<div class="flex items-center">
	<Label>{m.serverSettingsPref_title()}</Label>
	<Info>{m.serverSettingsPref_content()}</Info>
	<Button color="primary" icon="mdi:server-cog" onclick={() => (dialogOpen = true)}>
		{m.serverSettingsPref_configure()}
	</Button>
</div>

<DedicatedServerSettingsDialog bind:open={dialogOpen} />
