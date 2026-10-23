<script lang="ts">
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import { expoOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';
	import Spinner from '../ui/Spinner.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { Backend, FetchEvent } from '$lib/types';
	import { SvelteSet } from 'svelte/reactivity';
	import { getLocale } from '$lib/paraglide/runtime';

	let backendsFetching = $state(new SvelteSet<Backend>());
	let modsFetched = $state(0);

	const lst = new Intl.ListFormat(getLocale(), {
		style: 'long',
		type: 'conjunction'
	});

	let backendsList = $derived(lst.format(Array.from(backendsFetching)));

	let unlisten: UnlistenFn | null = null;

	onMount(() => {
		listen<FetchEvent>('fetch_event', (evt) => {
			switch (evt.payload.type) {
				case 'start':
					backendsFetching.add(evt.payload.backend);
					break;

				case 'progress':
					modsFetched += evt.payload.mods;
					break;

				case 'done':
					backendsFetching.delete(evt.payload.backend);
					if (backendsFetching.size === 0) {
						modsFetched = 0;
					}
					break;
			}
		}).then((fn) => (unlisten = fn));

		return () => {
			unlisten?.();
		};
	});
</script>

{#if backendsFetching.size > 0}
	<div
		class="text-primary-500 border-primary-200 dark:text-primary-400 dark:bg-primary-900 dark:border-primary-800 bg100 flex w-full items-center border-t px-4 py-2 text-sm"
		transition:slide={{ duration: 50, easing: expoOut }}
	>
		<Spinner />
		<span class="ml-2">
			{m.fetchModsBar_content({ backends: backendsList })}
			{#if modsFetched > 0}
				{modsFetched}
			{/if}
		</span>
	</div>
{/if}
