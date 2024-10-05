<script lang="ts">
	import Checkbox from '$lib/components/Checkbox.svelte';
	import Label from '$lib/components/Label.svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	export let label: string;
	export let disableMessage: string | null = null;

	export let value: boolean;
	export let set: (value: boolean) => Promise<void>;

	async function onValueChanged(newValue: boolean) {
		if (!newValue && disableMessage) {
			let confirmed = await confirm(disableMessage);
			if (!confirmed) {
				value = true;
				return;
			}
		}

		await set(newValue);
		value = newValue;
	}
</script>

<div class="my-1 flex items-center">
	<Label text={label}>
		<slot />
	</Label>

	<Checkbox bind:value {onValueChanged} />
</div>
