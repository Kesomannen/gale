<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';

	import PathField from '$lib/components/PathField.svelte';
	import { Button } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import { sentenceCase } from '$lib/util';

	export let label: string;
	export let type: 'dir' | 'file';
	export let canClear: boolean = false;

	export let value: string | null;
	export let set: (value: string | null) => Promise<void>;

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

<PathField {label} {value} onClick={browse} icon={type === 'file' ? 'mdi:file' : 'mdi:folder'}>
	<slot />

	<svelte:fragment slot="field">
		{#if canClear}
			<Button.Root
				class="absolute right-1 rounded p-1 text-lg text-slate-400 hover:bg-slate-800 hover:text-slate-300"
				on:click={(evt) => {
					evt.stopPropagation();
					value = null;
					set(null);
				}}
			>
				<Icon icon="mdi:close" />
			</Button.Root>
		{/if}
	</svelte:fragment>
</PathField>
