<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { Button } from 'bits-ui';

	export let entryId: ConfigEntryId;
	export let onReset: (value: ConfigValue) => void;

	function shouldConfirm(value: ConfigValue) {
		switch (value.type) {
			case 'string':
				return true;
			case 'float':
				return true;
			case 'int':
				return true;
			default:
				return false;
		}
	}

	async function onClick() {
		if (shouldConfirm(entryId.entry.value)) {
			let confirmed = await confirm(`Are you sure you want to reset ${entryId.entry.name}?`);
			if (!confirmed) return;
		}

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
	class="ml-1 rounded-lg p-1.5 text-xl text-slate-400 hover:bg-slate-700 hover:text-slate-300"
	on:click={onClick}
>
	<Icon icon="mdi:refresh" />
</Button.Root>
