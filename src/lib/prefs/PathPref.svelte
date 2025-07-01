<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';

	import PathField from '$lib/components/PathField.svelte';
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';
	import { toSentenceCase } from 'js-convert-case';

	type Props = {
		label: string;
		type: 'dir' | 'file';
		canClear?: boolean;
		value: string | null;
		set: (value: string | null) => Promise<void>;
		children?: Snippet;
	};

	let { label, type, canClear = false, value = $bindable(), set, children }: Props = $props();

	function browse() {
		open({
			defaultPath: value ?? undefined,
			title: 'Select ' + toSentenceCase(label),
			directory: type === 'dir'
		}).then(async (result) => {
			if (result === null) return;
			await set(result);
			value = result;
		});
	}
</script>

<PathField {label} {value} onclick={browse} icon={type === 'file' ? 'mdi:file' : 'mdi:folder'}>
	{@render children?.()}

	{#snippet field()}
		{#if canClear === true}
			<button
				class="text-primary-400 hover:bg-primary-800 hover:text-primary-300 absolute right-2 rounded-sm p-1 text-lg"
				onclick={(evt) => {
					evt.stopPropagation();
					value = null;
					set(null);
				}}
			>
				<Icon icon="mdi:close" />
			</button>
		{/if}
	{/snippet}
</PathField>
