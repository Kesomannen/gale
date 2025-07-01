<script lang="ts">
	import Checkbox from '$lib/components/Checkbox.svelte';
	import Info from '$lib/components/Info.svelte';
	import Label from '$lib/components/Label.svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	type Props = {
		label: string;
		disableMessage?: string | null;
		value: boolean;
		set: (value: boolean) => Promise<void>;
		children?: import('svelte').Snippet;
	};

	let { label, disableMessage = null, value = $bindable(), set, children }: Props = $props();

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
	<Label>
		{label}
	</Label>

	<Info>
		{@render children?.()}
	</Info>

	<Checkbox bind:checked={value} onCheckedChange={onValueChanged} />
</div>
