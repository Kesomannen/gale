<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import * as api from '$lib/api';
	import type { InstallProgress } from '$lib/types';
	import { formatTime, shortenFileSize } from '$lib/util';

	import { listen } from '@tauri-apps/api/event';

	import { Progress } from 'bits-ui';
	import { onMount } from 'svelte';
	import profiles from '$lib/state/profile.svelte';

	let open = $state(false);

	let progress: InstallProgress = $state({
		durationSecs: 0,
		totalProgress: 0,
		installedMods: 0,
		totalMods: 0,
		currentName: '',
		canCancel: false,
		task: {
			kind: 'installing'
		}
	});

	let currentName = $derived(progress.currentName.replace('_', ' '));
	let estimatedTimeLeft = $derived.by(() => {
		if (progress.durationSecs < 1) return '---';

		let estimatedSeconds = progress.durationSecs * (1 / progress.totalProgress - 1);
		if (estimatedSeconds < 60) return 'less than a minute';

		return formatTime(estimatedSeconds);
	});

	onMount(() => {
		listen<InstallProgress>('install_progress', (event) => {
			progress = event.payload;

			switch (progress.task.kind) {
				case 'done':
					progress.totalProgress = 1;
					progress.installedMods = progress.totalMods;
					profiles.refresh();
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

<Dialog
	bind:open
	title="Installing mods ({progress.installedMods}/{progress.totalMods})"
	canClose={progress.canCancel}
	onclose={() => api.profile.install.cancel()}
	confirmClose={{
		message: 'Are you sure you want to abort the installation?'
	}}
>
	<div class="text-primary-400">
		{#if progress.task.kind == 'done'}
			Done!
		{:else}
			<div>
				{#if progress.task.kind == 'downloading'}
					Downloading {currentName} ({shortenFileSize(
						progress.task.payload.downloaded
					)}/{shortenFileSize(progress.task.payload.total)})
				{:else if progress.task.kind == 'extracting'}
					Extracting {currentName}
				{:else if progress.task.kind == 'installing'}
					Installing {currentName}
				{/if}
			</div>

			<div>
				Estimated time remaining: {estimatedTimeLeft}
			</div>
		{/if}
	</div>

	<Progress.Root
		value={progress.totalProgress}
		max={1}
		class="bg-primary-900 relative mt-2 h-4 w-full overflow-hidden rounded-full"
	>
		<div
			class="bg-accent-600 absolute top-0 left-0 h-full rounded-l-full transition-all"
			style="width: {progress.totalProgress * 100}%"
		></div>
	</Progress.Root>
</Dialog>
