<script lang="ts">
	import Popup from '$lib/Popup.svelte';

	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

	import { Dialog, Progress } from 'bits-ui';

	export let modName: string;
	export let open: boolean;

	let unsubscribe: UnlistenFn | null = null;

	let current = 0;
	let total = 0;

	$: {
		if (open) {
			listen<number[]>('install_progress', (event) => {
				current = event.payload[0];
				total = event.payload[1];
			}).then((unsub) => {
				unsubscribe = unsub;
			});
		} else if (unsubscribe) {
			unsubscribe();
			unsubscribe = null;
		}
	}
</script>

<Popup title="Installing {modName} and dependencies" bind:open canClose={false}>
	<Dialog.Description class="text-slate-400">
		Installed {current} of {total}
	</Dialog.Description>
	<Progress.Root
		value={current}
		max={total}
		class="relative w-full h-4 mt-2 bg-gray-900 rounded-full overflow-hidden"
	>
		<div
			class="absolute top-0 left-0 h-full bg-green-600 rounded-full transition-all"
			style="width: {(current / total) * 100}%"
		/>
	</Progress.Root>
</Popup>
