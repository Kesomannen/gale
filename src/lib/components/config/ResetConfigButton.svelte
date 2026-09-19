<script lang="ts">
	import * as api from '$lib/api';
	import config from '$lib/state/config.svelte';
	import type { ConfigEntryId, ConfigValue } from '$lib/types';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import ResetButton from '$lib/components/ui/ResetButton.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		entryId: ConfigEntryId;
		onReset: (value: ConfigValue) => void;
	};

	let { entryId = $bindable(), onReset }: Props = $props();

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
			let confirmed = await confirm(m.resetConfigButton_confirm({ name: entryId.entry.name }));
			if (!confirmed) return;
		}

		if (config.profileId === null) return;
		let result = await api.config.resetEntry(entryId, config.profileId);

		entryId.entry.value = result;
		onReset(result);
	}
</script>

<ResetButton {onclick} class="ml-1" />
