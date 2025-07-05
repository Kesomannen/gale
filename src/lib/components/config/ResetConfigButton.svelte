<script lang="ts">
	import * as api from '$lib/api';
	import type { ConfigEntryId, ConfigValue } from '$lib/types';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import ResetButton from '$lib/components/ui/ResetButton.svelte';

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

<ResetButton disabled={locked} {onclick} class="ml-1" />
