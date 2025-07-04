<script lang="ts">
	import * as api from '$lib/api';
	import type { ConfigEntryId, ConfigValue } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

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

	async function onclick() {
		if (shouldConfirm(entryId.entry.value)) {
			let confirmed = await confirm(`Are you sure you want to reset ${entryId.entry.name}?`);
			if (!confirmed) return;
		}

		let result = await api.config.resetEntry(entryId);

		entryId.entry.value = result;
		onReset(result);
	}
</script>

<button
	class="text-primary-400 disabled:text-primary-500 enabled:hover:bg-primary-700 enabled:hover:text-primary-300 ml-1 rounded-lg p-1.5 text-xl disabled:cursor-not-allowed"
	disabled={locked}
	{onclick}
>
	<Icon icon="mdi:refresh" />
</button>
