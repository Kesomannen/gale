<script lang="ts">
	import Popup from '$lib/Popup.svelte';
	import type { InstallProgress, InstallProgressPayload } from '$lib/models';
	import { shortenFileSize } from '$lib/util';

	import { listen } from '@tauri-apps/api/event';

	import { Dialog, Progress } from 'bits-ui';
	import { onMount } from 'svelte';

	let open = false;

	let progress: InstallProgress = {
		installedMods: 0,
		totalMods: 0,
		downloadedBytes: 0,
		totalBytes: 0,
		currentModName: '',
		currentTask: {
			type: 'installing',
		}
	};

	onMount(() => {
		listen<InstallProgressPayload>('install_progress', (event) => {
			let payload = event.payload;

			switch (payload.type) {
				case 'done':
					progress.downloadedBytes = progress.totalBytes;
					progress.installedMods = progress.totalMods;
					setTimeout(() => {
						open = false;
					}, 500);
					break;

				case 'inProgress':
					progress = payload.content;
					open = true;
					break;
				
				case 'error':
					open = false;
					break;

				default:
					
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
		{#if progress.downloadedBytes === progress.totalBytes}
			Done!
		{:else if progress.currentTask.type == 'downloading'}
			Downloading {progress.currentModName} ({shortenFileSize(progress.currentTask.content.downloaded)}/{shortenFileSize(progress.currentTask.content.total)})
		{:else if progress.currentTask.type == 'extracting'}
			Extracting {progress.currentModName}
		{:else if progress.currentTask.type == 'installing'}
			Installing {progress.currentModName}
		{/if}
	</Dialog.Description>

	<Progress.Root
		value={progress.downloadedBytes}
		max={progress.totalBytes}
		class="relative w-full h-4 mt-2 bg-gray-900 rounded-full overflow-hidden"
	>
		<div
			class="absolute top-0 left-0 h-full bg-green-600 rounded-full transition-all"
			style="width: {(progress.downloadedBytes / progress.totalBytes) * 100}%"
		/>
	</Progress.Root>
</Popup>
