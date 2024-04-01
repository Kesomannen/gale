<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';

	export let entryId: ConfigEntryId;
	export let onReset: (value: ConfigValue) => void;

	function onClick() {
		invokeCommand<ConfigValue>('reset_config_entry', {
			file: entryId.file.name,
			section: entryId.section.name,
			entry: entryId.entry.name
		}).then((result) => onReset(result));
	}
</script>

<Button.Root
	class="text-slate-400 text-lg hover:text-slate-300 hover:bg-gray-700 p-1 ml-1 rounded"
	on:click={onClick}
>
	<Icon icon="mdi:refresh" />
</Button.Root>
