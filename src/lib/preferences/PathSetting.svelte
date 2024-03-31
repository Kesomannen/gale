<script lang="ts">
	import { open } from '@tauri-apps/api/dialog';

	import { invokeCommand } from '$lib/invoke';
	import { onMount } from 'svelte';
	import type { PrefEntry } from '$lib/models';
	import PathField from '$lib/PathField.svelte';

	export let label: string;
	export let name: string;
	export let type: 'exe' | 'dir';

	let value: string | null = null;

	onMount(async () => {
		value = (await invokeCommand<PrefEntry>('get_pref', { name })).value.content as string;
	});

	function browse() {
		open({
			defaultPath: value ?? undefined,
			filters: type === 'exe' ? [{ name: 'Executable', extensions: ['exe'] }] : undefined,
			title: 'Select ' + name,
			multiple: false,
			directory: type === 'dir'
		}).then((result) => {
			if (result === null) return;

			value = result as string;
			invokeCommand('set_pref', { entry: { name, value: { content: value, type: 'Path' }}});
		});
	}
</script>

<PathField {label} {value} onClick={browse} icon={type === 'exe' ? 'mdi:file' : 'mdi:folder'} />