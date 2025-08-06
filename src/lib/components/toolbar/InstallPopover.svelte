<script lang="ts">
	import profiles from '$lib/state/profile.svelte';
	import type { InstallEvent, InstallTask } from '$lib/types';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { Popover, Progress } from 'bits-ui';
	import { onDestroy, onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { fade, fly, scale } from 'svelte/transition';
	import { expoInOut, quadOut } from 'svelte/easing';
	import { dropIn, dropOut } from '$lib/transitions';
	import { Tween } from 'svelte/motion';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import * as api from '$lib/api';

	let shown = $state(false);
	let showCancel = $state(false);
	let open = $state(false);

	let totalMods = $state(0);
	let totalBytes = $state(0);

	let completedMods = $state(0);
	let completedBytes = $state(0);

	let name: string | null = $state(null);
	let task: InstallTask | null = $state(null);

	let hideTimeout: number | null = $state(null);

	let taskText = $derived(
		task
			? {
					download: 'Downloading ',
					extract: 'Extracting ',
					install: 'Installing '
				}[task] + name!.replace(/_/g, ' ')
			: null
	);

	let progress = $derived(totalBytes === 0 ? 0 : completedBytes / totalBytes);
	let shownProgress = new Tween(0);

	$effect(() => {
		shownProgress.target = progress;
	});

	let unlisten: UnlistenFn | null = null;

	onMount(async () => {
		unlisten = await listen<InstallEvent>('install_event', (event) => {
			switch (event.payload.type) {
				case 'show':
					shown = true;
					open = true;
					showCancel = true;

					if (hideTimeout) {
						clearTimeout(hideTimeout);
					}

					break;

				case 'hide':
					showCancel = false;

					let hideDelay = 0;
					if (event.payload.reason === 'done') {
						hideDelay = 500;

						taskText = 'Finishing up...';
						shownProgress.set(1, { duration: 250, easing: expoInOut });
					}

					hideTimeout = setTimeout(() => {
						open = false;

						totalMods = 0;
						totalBytes = 0;

						completedMods = 0;
						completedBytes = 0;

						name = null;
						task = null;

						hideTimeout = setTimeout(() => {
							shown = false;

							shownProgress.set(0, { delay: 50, duration: 0 });

							hideTimeout = null;
						}, 100);
					}, hideDelay);

					break;

				case 'addCount':
					totalMods += event.payload.mods;
					totalBytes += event.payload.bytes;

					shownProgress.set(progress, { duration: 0 });

					if (event.payload.mods > 0) {
						open = true;
					}
					break;

				case 'addProgress':
					completedMods += event.payload.mods;
					completedBytes += event.payload.bytes;
					break;

				case 'setTask':
					name = event.payload.name;
					task = event.payload.task;
					break;
			}
		});
	});

	onDestroy(() => {
		unlisten?.();
	});

	async function cancel() {
		open = false;
		showCancel = false;
		await api.profile.install.cancelAll();
	}
</script>

<Popover.Root bind:open>
	<Popover.Trigger class="hover:bg-primary-800 text-accent-500 my-auto rounded-md p-1.5 text-xl">
		{#if shown}
			<div in:scale={{ start: 2, duration: 250, easing: quadOut }}>
				<Icon icon="mdi:download" class="animate-pulse" />
			</div>
		{/if}
	</Popover.Trigger>
	<Popover.Content forceMount>
		{#snippet child({ wrapperProps, props, open })}
			<div {...wrapperProps}>
				{#if open}
					<div
						{...props}
						class="border-primary-600 bg-primary-800 z-10 w-80 rounded-lg border px-6 py-4 shadow-xl"
						in:fly={dropIn}
						out:fade={dropOut}
					>
						<div class="text-primary-300 flex items-center justify-between font-semibold">
							<div>Installing mods... ({completedMods}/{totalMods})</div>
							{#if showCancel}
								<IconButton label="Cancel" icon="mdi:cancel" color="red" onclick={cancel} />
							{/if}
						</div>

						{#if task && name}
							<div class="text-primary-400 text-sm">
								{taskText}
							</div>
						{/if}

						<Progress.Root
							value={shownProgress.current}
							max={1}
							class="bg-primary-900 relative mt-2 h-4 w-full overflow-hidden rounded-full"
						>
							<div
								class="bg-accent-600 absolute top-0 left-0 h-full rounded-l-full"
								style="width: {shownProgress.current * 100}%"
							></div>
						</Progress.Root>
					</div>
				{/if}
			</div>
		{/snippet}
	</Popover.Content>
</Popover.Root>
