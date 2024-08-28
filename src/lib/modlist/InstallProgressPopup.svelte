<script lang="ts">
	import { T, t } from '$i18n';
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { InstallProgress } from '$lib/models';
	import { formatTime, shortenFileSize } from '$lib/util';

	import { listen } from '@tauri-apps/api/event';

	import { Dialog, Progress } from 'bits-ui';
	import { onMount } from 'svelte';

	let open = false;

	let progress: InstallProgress = {
		totalProgress: 0,
		installedMods: 0,
		totalMods: 0,
		currentName: '',
		canCancel: false,
		task: {
			kind: 'installing'
		}
	};

	onMount(() => {
		listen<InstallProgress>('install_progress', (event) => {
			progress = event.payload;

			switch (progress.task.kind) {
				case 'done':
					progress.totalProgress = 1;
					progress.installedMods = progress.totalMods;
					setTimeout(() => {
						open = false;
					}, 250);
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
	title="{t['Installing mods']} ({progress.installedMods}/{progress.totalMods})"
	canClose={progress.canCancel}
	bind:open
	confirmClose={{
		title: t["Abort installation"],
		message: t["Abort installation description"]
	}}
	onClose={() => {
		invokeCommand('cancel_install')
	}}
>
	<Dialog.Description class="text-slate-400">
		{#if progress.task.kind == 'done'}
			{t["Done ex"]}
		{:else if progress.task.kind == 'downloading'}
			{T(t["Install progress downloading"], {"name": progress.currentName})} ({shortenFileSize(progress.task.payload.downloaded)}/{shortenFileSize(progress.task.payload.total)})
		{:else if progress.task.kind == 'extracting'}
			{T(t["Install progress extracting"], {"name": progress.currentName})}
		{:else if progress.task.kind == 'installing'}
			{T(t["Install progress installing"], {"name": progress.currentName})}
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
