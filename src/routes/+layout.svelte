<script lang="ts">
	import '../app.css';

	import Menubar from '$lib/menu/Menubar.svelte';
	import Contextbar from '$lib/menu/Contextbar.svelte';

	import { errors, removeError } from '$lib/invoke';

	import { Button } from 'bits-ui';
	import Icon from '@iconify/svelte';

	import { expoOut } from 'svelte/easing';
	import { fade, fly, slide } from 'svelte/transition';
	import { onDestroy, onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import NavbarLink from '$lib/menu/NavbarLink.svelte';
	import InstallProgressPopup from '$lib/modlist/InstallProgressPopup.svelte';
	import WelcomePopup from '$lib/menu/WelcomePopup.svelte';

	let status: string | undefined;
	let unlisten: UnlistenFn | undefined;

	onMount(async () => {
		unlisten = await listen<string | undefined>('status_update', (evt) => {
			status = evt.payload;
		});
	});

	onDestroy(() => {
		if (unlisten) {
			unlisten();
		}
	});
</script>

<main
	class="h-screen overflow-hidden flex flex-col rounded-lg border border-gray-600 bg-gray-800 relative"
	on:contextmenu={(e) => {
		if (window.location.hostname === 'tauri.localhost') {
			e.preventDefault();
		}
	}}
>
	<Menubar />
	<Contextbar />

	<div class="flex flex-grow overflow-hidden relative">
		<div
			class="flex flex-col gap-1 items-center p-2 w-14 bg-gray-900 border-r border-gray-600 flex-shrink-0"
		>
			<NavbarLink to="/" icon="mdi:home" tooltip="Home page" />
			<NavbarLink to="/profile" icon="mdi:account-circle" tooltip="Manage profile" />
			<NavbarLink to="/mods" icon="material-symbols:browse" tooltip="Browse mods" />
			<NavbarLink to="/config" icon="mdi:file-cog" tooltip="Edit mod config" />
			<NavbarLink to="/modpack" icon="mdi:package-variant" tooltip="Export modpack" />
			<div class="flex-grow" />
			<NavbarLink to="/prefs" icon="mdi:settings" tooltip="Edit manager settings" />
		</div>

		<slot />
	</div>

	{#if status}
		<div
			class="w-full flex items-center px-3 py-1 text-sm border-t border-gray-700 text-slate-400"
			transition:slide={{ duration: 200, easing: expoOut }}
		>
			<Icon icon="mdi:loading" class="animate-spin" />
			<span class="ml-2">{status}</span>
		</div>
	{/if}

	<div
		class="flex flex-col-reverse justify-end max-w-[50rem] xl:max-w-[90rem]
          bottom-0 right-0 gap-1 absolute z-10 p-2"
	>
		{#each $errors as error, i}
			<div
				class="flex items-start bg-red-600 rounded-md p-1.5 xl:text-lg xl:p-2"
				in:slide={{ duration: 150, easing: expoOut }}
				out:fade={{ duration: 100 }}
			>
				<div class="flex-grow px-2 mt-auto mr-3">
					<span class="text-red-200">{error.name} -</span>
					<span class="text-white font-medium ml-1">{error.message}</span>
				</div>

				<Button.Root
					class="p-1 hover:bg-red-500 rounded-sm"
					on:click={() => writeText('`' + error.name + ' - ' + error.message + '`')}
				>
					<Icon icon="mdi:clipboard-text" class="text-slate-100 text-lg" />
				</Button.Root>

				<Button.Root class="p-1 hover:bg-red-500 rounded-md" on:click={() => removeError(i)}>
					<Icon icon="mdi:close" class="text-slate-100 text-lg" />
				</Button.Root>
			</div>
		{/each}
	</div>
</main>

<InstallProgressPopup />
<WelcomePopup />
