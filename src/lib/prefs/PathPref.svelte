<script lang="ts">
	import { open } from '@tauri-apps/api/dialog';

	import { invokeCommand } from '$lib/invoke';
	import PathField from '$lib/components/PathField.svelte';
	import type { PrefValue } from '$lib/models';

	export let label: string;
	export let key: string;
	export let type: 'dir' | 'file';

	let value: string | null;

	$: getValue(key);

	function browse() {
		open({
			defaultPath: value ?? undefined,
			title: 'Select ' + key,
			directory: type === 'dir'
		}).then(async (result) => {
			if (result === null) return;

			value = result as string;
			setValue(value);
		});
	}

	function setValue(value: string | null) {
		invokeCommand('set_pref', { key, value });
	};

	async function getValue(key: string) {
		value = await invokeCommand<PrefValue | null>('get_pref', { key }) as string;
	}
</script>

<PathField {label} {value} onClick={browse} icon={type === 'file' ? 'mdi:file' : 'mdi:folder'}>
	<slot />
</PathField>