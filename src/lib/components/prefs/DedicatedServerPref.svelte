<script lang="ts">
	import Info from '$lib/components/ui/Info.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import DedicatedServerDialog from '$lib/components/dialogs/DedicatedServerDialog.svelte';
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';

	let dialogOpen = $state(false);
</script>

<div class="flex items-center">
	<Label>{m.dedicatedServerPref_title()}</Label>

	<Info>
		{m.dedicatedServerPref_content()}
	</Info>

	<button
		class="group enabled:hover:border-primary-400 text-primary-600 dark:bg-primary-900 dark:enabled:hover:border-primary-500 dark:text-primary-300 bg-primary-100 relative flex grow items-center gap-2 truncate rounded-lg border border-transparent px-3 py-1 disabled:cursor-not-allowed disabled:opacity-70"
		onclick={() => (dialogOpen = true)}
	>
		<Icon icon="mdi:server" />

		<div class="truncate">
			{m.dedicatedServerPref_configure()}
		</div>
	</button>
</div>

<DedicatedServerDialog bind:open={dialogOpen} />
