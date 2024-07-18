<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';

	import { invokeCommand } from '$lib/invoke';
	import PathField from '$lib/components/PathField.svelte';
	import { Button } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import { sentenceCase } from '$lib/util';

	export let label: string;
	export let type: 'dir' | 'file';
	export let canClear: boolean = false;
	
	export let value: string | null;
	export let set: (value: string | null) => void;

	function browse() {
		open({
			defaultPath: value ?? undefined,
			title: 'Select ' + sentenceCase(label),
			directory: type === 'dir'
		}).then(async (result) => {
      		if (result === null) return;

			value = result as string;
			set(result as string);
		});
	}
</script>

<PathField {label} {value} onClick={browse} icon={type === 'file' ? 'mdi:file' : 'mdi:folder'}>
	<slot />

	<svelte:fragment slot="field">
		{#if canClear}
			<Button.Root
				class="absolute right-1 rounded-md text-xl text-slate-500 p-1
		 		hover:bg-gray-800 hover:text-slate-400"
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
