<script lang="ts">
	import '../app.css';

	import Menubar from '$lib/menu/Menubar.svelte';
	import Contextbar from '$lib/menu/Contextbar.svelte';

	import { errors, removeError } from '$lib/invoke';

	import { Button } from 'bits-ui';
	import Icon from '@iconify/svelte';

	import { expoOut } from 'svelte/easing';
	import { fade, slide } from 'svelte/transition';
	import { onDestroy, onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import NavbarLink from '$lib/menu/NavbarLink.svelte';
	import InstallProgressPopup from '$lib/modlist/InstallProgressPopup.svelte';
	import WelcomePopup from '$lib/menu/WelcomePopup.svelte';

	let status: string | null = null;
	let unlisten: UnlistenFn | undefined;

	onMount(async () => {
		unlisten = await listen<string | null>('status_update', (evt) => {
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
	class="relative flex flex-col overflow-hidden bg-slate-800"
	on:contextmenu={(e) => {
		if (window.location.hostname === 'tauri.localhost') {
			e.preventDefault();
		}
	}}
>
	<Menubar />
	<Contextbar />

	<div class="relative flex flex-grow overflow-hidden">
		<nav
			class="flex w-14 flex-shrink-0 flex-col items-center gap-1 border-r border-gray-600 bg-gray-900 p-2"
		>
			<NavbarLink to="/" icon="mdi:account-circle" tooltip="Manage profile" />
			<NavbarLink to="/browse" icon="mdi:store-search" tooltip="Browse Thunderstore mods" />
			<NavbarLink to="/config" icon="mdi:file-cog" tooltip="Edit mod config" />
			<NavbarLink to="/modpack" icon="mdi:package-variant" tooltip="Export modpack" />
			<NavbarLink to="/prefs" icon="mdi:settings" tooltip="Edit manager settings" />
		</nav>

		<slot />
	</div>

	{#if status !== null}
		<div
			class="flex w-full items-center border-t border-gray-600 px-3 py-1 text-sm text-slate-400"
			transition:slide={{ duration: 200, easing: expoOut }}
		>
			<Icon icon="mdi:loading" class="animate-spin" />
			<span class="ml-2">{status}</span>
		</div>
	{/if}

	<div
		class="absolute bottom-0 right-0 z-10 flex max-w-[50rem] flex-col-reverse justify-end gap-1 p-2 xl:max-w-[90rem]"
	>
		{#each $errors as error, i}
			<div
				class="flex items-start rounded-md bg-red-600 p-1.5 xl:p-2 xl:text-lg"
				in:slide={{ duration: 150, easing: expoOut }}
				out:fade={{ duration: 100 }}
			>
				<div class="mr-3 mt-auto flex-grow px-2">
					<span class="text-red-200">{error.name} -</span>
					<span class="ml-1 font-medium text-white">{error.message}</span>
				</div>

				<Button.Root
					class="rounded-sm p-1 hover:bg-red-500"
					on:click={() => writeText('`' + error.name + ' - ' + error.message + '`')}
				>
					<Icon icon="mdi:clipboard-text" class="text-lg text-slate-100" />
				</Button.Root>

				<Button.Root class="rounded-md p-1 hover:bg-red-500" on:click={() => removeError(i)}>
					<Icon icon="mdi:close" class="text-lg text-slate-100" />
				</Button.Root>
			</div>
		{/each}
	</div>
</main>

<InstallProgressPopup />
<WelcomePopup />

<style lang="postcss">
	:global(body) {
		overflow: hidden;
		position: fixed;
		width: 100vw;
		height: 100vh;
	}

	main {
		height: calc(100vh - 1px);
	}

	:global(div) {
		scrollbar-color: theme(colors.gray.500) theme(colors.gray.800);
	}
</style>
