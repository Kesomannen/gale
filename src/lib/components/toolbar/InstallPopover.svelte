<script lang="ts">
	import profiles from '$lib/state/profile.svelte';
	import type { InstallEvent } from '$lib/types';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { Popover, Progress } from 'bits-ui';
	import { onDestroy, onMount } from 'svelte';
	import Spinner from '../ui/Spinner.svelte';
	import Icon from '@iconify/svelte';
	import { fade, fly, scale } from 'svelte/transition';
	import { expoOut, quadOut } from 'svelte/easing';
	import { pushInfoToast } from '$lib/toast';
	import { shortenFileSize } from '$lib/util';
	import { dropIn, dropOut } from '$lib/transitions';
	import { Spring, Tween } from 'svelte/motion';

	let shown = $state(false);
	let open = $state(false);

	let totalMods = $state(0);
	let totalBytes = $state(0);

	let completedMods = $state(0);
	let completedBytes = $state(0);

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
					break;

				case 'hide':
					pushInfoToast({
						message: `Installed ${totalMods} ${totalMods === 1 ? 'mod' : 'mods'} (${shortenFileSize(totalBytes)}).`
					});

					shownProgress.set(1, { duration: 100, easing: expoOut });

					setTimeout(() => {
						open = false;

						totalMods = 0;
						totalBytes = 0;

						completedMods = 0;
						completedBytes = 0;

						setTimeout(() => {
							shown = false;
						}, 100);
					}, 250);

					break;

				case 'addCount':
					totalMods += event.payload.mods;
					totalBytes += event.payload.bytes;

					if (event.payload.mods > 0) {
						open = true;
					}
					break;

				case 'addProgress':
					completedMods += event.payload.mods;
					completedBytes += event.payload.bytes;

					if (event.payload.mods > 0) {
						profiles.refresh();
					}
					break;
			}
		});
	});

	onDestroy(() => {
		unlisten?.();
	});
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
						class="border-primary-600 bg-primary-800 z-30 w-80 rounded-lg border p-6 shadow-xl"
						in:fly={dropIn}
						out:fade={dropOut}
					>
						<div class="text-primary-300 font-semibold">
							Installing mods... ({completedMods}/{totalMods})
						</div>

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
