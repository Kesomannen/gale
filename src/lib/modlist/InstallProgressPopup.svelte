<script lang="ts">
	import Popup from '$lib/Popup.svelte';
	import type { InstallProgress } from '$lib/models';
	import { shortenFileSize } from '$lib/util';

	import { listen } from '@tauri-apps/api/event';

	import { Dialog, Progress } from 'bits-ui';
	import { onMount } from 'svelte';

	let open = false;

	let progress: InstallProgress = {
		totalProgress: 0,
		installedMods: 0,
		totalMods: 0,
		currentName: '',
		task: {
			kind: 'installing'
		}
	};

	onMount(() => {
		listen<InstallProgress>('install_progress', (event) => {
			progress = event.payload;
			console.log(progress);

			switch (progress.task.kind) {
				case 'done':
					progress.totalProgress = 1;
					progress.installedMods = progress.totalMods;
					setTimeout(() => {
						open = false;
					}, 500);
					break;
				
				case 'error':
					open = false;
					break;

				default:
					open = true;
					break;
			}
		});
	});
</script>

<Popup
	title="Installing mods ({progress.installedMods}/{progress.totalMods})"
	bind:open
	canClose={false}
>
	<Dialog.Description class="text-slate-400">
		{#if progress.task.kind == 'done'}
			Done!
		{:else if progress.task.kind == 'downloading'}
			Downloading {progress.currentName} ({shortenFileSize(progress.task.payload.downloaded)}/{shortenFileSize(progress.task.payload.total)})
		{:else if progress.task.kind == 'extracting'}
			Extracting {progress.currentName}
		{:else if progress.task.kind == 'installing'}
			Installing {progress.currentName}
		{/if}
	</Dialog.Description>

	<Progress.Root
		value={progress.totalProgress}
		max={1}
		class="relative w-full h-4 mt-2 bg-gray-900 rounded-full overflow-hidden"
	>
		<div
			class="absolute top-0 left-0 h-full bg-green-600 rounded-full transition-all"
			style="width: {progress.totalProgress * 100}%"
		/>
	</Progress.Root>
</Popup>
