<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { ConfigEntryId, ConfigValue } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { Button } from 'bits-ui';

	type Props = {
		entryId: ConfigEntryId;
		locked: boolean;
		onReset: (value: ConfigValue) => void;
	};

	let { entryId = $bindable(), locked, onReset }: Props = $props();

	function shouldConfirm(value: ConfigValue) {
		switch (value.type) {
			case 'string':
			case 'float':
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
	class="text-primary-400 disabled:text-primary-500 enabled:hover:bg-primary-700 enabled:hover:text-primary-300 ml-1 rounded-lg p-1.5 text-xl disabled:cursor-not-allowed"
	disabled={locked}
	on:click={onClick}
>
	<Icon icon="mdi:refresh" />
</Button.Root>
