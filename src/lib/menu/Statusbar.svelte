<script lang="ts">
	import Icon from '@iconify/svelte';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import { expoOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';

	let status: string | null = null;

	onMount(() => {
		listen<string | null>('status_update', (evt) => {
			status = evt.payload;
		});
	});
</script>

{#if status !== null}
	<div
		class="border-primary-600 text-primary-400 flex w-full items-center border-t px-3 py-1 text-sm"
		transition:slide={{ duration: 200, easing: expoOut }}
	>
		<Icon icon="mdi:loading" class="animate-spin" />
		<span class="ml-2">{status}</span>
	</div>
{/if}
