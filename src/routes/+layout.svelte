<script lang="ts">
	import '../app.css';

	import Menubar from '$lib/menu/Menubar.svelte';
	import Contextbar from '$lib/menu/Contextbar.svelte';
	
	import { errors, removeError } from '$lib/invoke';

	import { Button } from 'bits-ui';
	import Icon from '@iconify/svelte';

	import { expoOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import NavbarLink from '$lib/menu/NavbarLink.svelte';

	let status: string | undefined;

	onMount(() => {
		listen<string | undefined>('status_update', (evt) => {
			status = evt.payload;
		})
	});
</script>

<main
	class="h-screen overflow-hidden flex flex-col rounded-lg border border-gray-600 bg-gray-800 relative"
>
	<Menubar />
	<Contextbar />

	<div class="flex flex-grow overflow-hidden">
		<div class="flex flex-col gap-1 items-center p-2 w-14 bg-gray-900 border-r border-gray-600 flex-shrink-0">
			<NavbarLink to="/mods" icon="material-symbols:browse" tooltip="Browse mods" />
			<NavbarLink to="/profile" icon="mdi:account-circle" tooltip="Manage profile" />
			<NavbarLink to="/config" icon="mdi:settings" tooltip="Edit mod config" />
		</div>

		<slot />
	</div>

	{#if status}
		<div class="w-full flex items-center px-3 py-1 text-sm border-t border-gray-700 text-slate-400">
			<Icon icon="mdi:loading" class="animate-spin" />
			<span class="ml-2">{status}</span>
		</div>
	{/if}

	<div class="bottom-0 right-0 w-full max-w-[50rem] p-2 gap-1 absolute flex flex-col-reverse">
		{#each $errors as error, i}
			<div
				class="bg-red-600 pl-4 pr-8 py-2 rounded-md relative" 
				transition:slide={{ duration: 200, easing: expoOut }}
			>
				<span class="text-red-200">Failed to execute '{error.name}' -</span>
				<span class="text-red-100 font-medium ml-1">{error.message}</span>

				<Button.Root class="absolute top-3 right-3" on:click={() => removeError(i)}>
					<Icon icon="mdi:close" class="text-red-200" />
				</Button.Root>
			</div>
		{/each}
	</div>
</main>
