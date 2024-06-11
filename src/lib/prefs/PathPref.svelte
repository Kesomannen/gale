<script lang="ts">
	import { open } from '@tauri-apps/api/dialog';

	import { invokeCommand } from '$lib/invoke';
	import PathField from '$lib/components/PathField.svelte';
	import type { PrefValue } from '$lib/models';
	import { Button } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import { sentenceCase } from '$lib/util';

	export let label: string;
	export let key: string;
	export let type: 'dir' | 'file';
	export let canClear: boolean = false;

	let value: string | null;

	$: getValue(key);

	function browse() {
		open({
			defaultPath: value ?? undefined,
			title: 'Select ' + sentenceCase(key),
			directory: type === 'dir'
		}).then(async (result) => {
      if (result === null) return;

			setValue(result as string);
		});
	}

	async function setValue(v: string | null) {
		value = v;
		await invokeCommand('set_pref', { key, value });
	}

	async function getValue(key: string) {
		value = (await invokeCommand<PrefValue | null>('get_pref', { key })) as string;
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
					setValue(null);
				}}
			>
				<Icon icon="mdi:close" />
			</Button.Root>
		{/if}
	</svelte:fragment>
</PathField>
