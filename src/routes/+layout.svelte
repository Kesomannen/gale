<script lang="ts">
	import '../app.css';

	import Menubar from '$lib/menu/Menubar.svelte';
	import Contextbar from '$lib/menu/Contextbar.svelte';

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
	import { clearToast, toasts } from '$lib/toast';
	import { refreshColor } from '$lib/theme';

	let status: string | null = null;
	let unlisten: UnlistenFn | undefined;

	onMount(async () => {
		refreshColor('accent');
		refreshColor('primary');

		unlisten = await listen<string | null>('status_update', (evt) => {
			status = evt.payload;
		});
	});
</script>

<main
	class="bg-primary-800 relative flex flex-col overflow-hidden"
	on:contextmenu={(evt) => {
		// hide context menu in release builds
		if (window.location.hostname === 'tauri.localhost') {
			evt.preventDefault();
		}
	}}
>
	<Menubar />
	<Contextbar />

	<div class="relative flex grow overflow-hidden">
		<nav class="border-primary-600 bg-primary-900 flex shrink-0 flex-col gap-1 border-r p-2.5">
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
			class="border-primary-600 text-primary-400 flex w-full items-center border-t px-3 py-1 text-sm"
			transition:slide={{ duration: 200, easing: expoOut }}
		>
			<Icon icon="mdi:loading" class="animate-spin" />
			<span class="ml-2">{status}</span>
		</div>
	{/if}

	<div
		class="absolute right-0 bottom-0 z-10 flex max-w-[50rem] flex-col items-end justify-end gap-1 p-2 xl:max-w-[90rem]"
	>
		{#each $toasts as toast, i}
			<div
				class="flex items-start overflow-hidden rounded-md p-1.5 xl:p-2 xl:text-lg {toast.type ===
				'error'
					? 'bg-red-600'
					: 'bg-accent-600'}"
				in:slide={{ duration: 150, easing: expoOut }}
				out:fade={{ duration: 100 }}
			>
				<div class="mt-auto mr-3 grow overflow-hidden px-2">
					{#if toast.name !== undefined}
						<span class={toast.type === 'error' ? 'text-red-200' : 'text-accent-200'}
							>{toast.name} -</span
						>
					{/if}

					<span class="font-medium break-words text-white">{toast.message}</span>
				</div>

				{#if toast.type === 'error'}
					<Button.Root
						class="rounded-xs p-1 hover:bg-red-500"
						on:click={() => writeText('`' + toast.name + ' - ' + toast.message + '`')}
					>
						<Icon icon="mdi:clipboard-text" class="text-primary-100 text-lg" />
					</Button.Root>
				{/if}

				<Button.Root
					class="rounded-md p-1 {toast.type === 'error'
						? 'hover:bg-red-500'
						: 'hover:bg-accent-500'}"
					on:click={() => clearToast(i)}
				>
					<Icon icon="mdi:close" class="text-primary-100 text-lg" />
				</Button.Root>
			</div>
		{/each}
	</div>
</main>

<InstallProgressPopup />
<WelcomePopup />
