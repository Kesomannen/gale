<script lang="ts">
	import { open } from '@tauri-apps/api/dialog';

	import { invokeCommand } from '$lib/invoke';
	import { onMount } from 'svelte';
	import PathField from '$lib/components/PathField.svelte';
	import type { PrefValue } from '$lib/models';

	export let label: string;
	export let key: string;
	export let type: 'exe' | 'dir';

	export let setValue: (value: string | null) => void = (value) => {
		invokeCommand('set_pref', { key, value });
	};

	export let getValue: () => Promise<string | null> = async () => {
		return await invokeCommand<PrefValue | null>('get_pref', { key }) as string;
	};

	let value: string | null = null;

	onMount(async () => {
		value = await getValue();
	});

	function browse() {
		open({
			defaultPath: value ?? undefined,
			filters: type === 'exe' ? [{ name: 'Executable', extensions: ['exe'] }] : undefined,
			title: 'Select ' + key,
			directory: type === 'dir'
		}).then((result) => {
			if (result === null) return;

			value = result as string;
			setValue(value);
		});
	}
</script>

<PathField {label} {value} onClick={browse} icon={type === 'exe' ? 'mdi:file' : 'mdi:folder'}>
	<slot />
</PathField>