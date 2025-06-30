<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';

	import PathField from '$lib/components/PathField.svelte';
	import { Button } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import { sentenceCase } from '$lib/util';

	type Props = {
		label: string;
		type: 'dir' | 'file';
		canClear?: boolean;
		value: string | null;
		set: (value: string | null) => Promise<void>;
		children?: import('svelte').Snippet;
	};

	let { label, type, canClear = false, value = $bindable(), set, children }: Props = $props();

	function browse() {
		open({
			defaultPath: value ?? undefined,
			title: 'Select ' + sentenceCase(label),
			directory: type === 'dir'
		}).then(async (result) => {
			if (result === null) return;
			await set(result);
			value = result;
		});
	}
</script>

<PathField {label} {value} on:click={browse} icon={type === 'file' ? 'mdi:file' : 'mdi:folder'}>
	{@render children?.()}

	{#snippet field()}
		{#if canClear}
			<Button.Root
				class="text-primary-400 hover:bg-primary-800 hover:text-primary-300 absolute right-2 rounded-sm p-1 text-lg"
				on:click={(evt) => {
					evt.stopPropagation();
					value = null;
					set(null);
				}}
			>
				<Icon icon="mdi:close" />
			</Button.Root>
		{/if}
	{/snippet}
</PathField>
