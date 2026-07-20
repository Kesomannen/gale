<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
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
		});
	});
</script>

{#if backendsFetching.size > 0}
	<div
		class="border-primary-600 text-primary-400 flex w-full items-center border-t px-3 py-1 text-sm"
		transition:slide={{ duration: 200, easing: expoOut }}
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
