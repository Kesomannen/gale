<script lang="ts">
	import Tooltip from '$lib/components/Tooltip.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';

	export let entryId: ConfigEntryId;
	export let onReset: (value: ConfigValue) => void;

	async function onClick() {
		let result = await invokeCommand<ConfigValue>('reset_config_entry', {
			file: entryId.file.relativePath,
			section: entryId.section.name,
			entry: entryId.entry.name
		});

		entryId.entry.value = result;
		onReset(result);
	}
</script>

<Button.Root
	class="ml-1 rounded-lg p-1.5 text-xl text-slate-400 hover:bg-gray-700 hover:text-slate-300"
	on:click={onClick}
>
	<Icon icon="mdi:refresh" />
</Button.Root>
