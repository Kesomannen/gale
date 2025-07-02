<script lang="ts">
	import '../app.css';

	import { Tooltip } from 'bits-ui';

	import Menubar from '$lib/menu/Menubar.svelte';
	import Contextbar from '$lib/menu/Contextbar.svelte';
	import Statusbar from '$lib/menu/Statusbar.svelte';
	import Toasts from '$lib/menu/Toasts.svelte';

	import { onMount, type Snippet } from 'svelte';
	import NavbarLink from '$lib/menu/NavbarLink.svelte';
	import { refreshColor, refreshFont } from '$lib/theme';
	import InstallModPopup from '$lib/modlist/InstallModPopup.svelte';
	import InstallProgressPopup from '$lib/modlist/InstallProgressPopup.svelte';
	import WelcomePopup from '$lib/menu/WelcomePopup.svelte';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();

	onMount(() => {
		refreshFont();
		refreshColor('accent');
		refreshColor('primary');

		// workaround for https://github.com/huntabyte/bits-ui/issues/1639
		setTimeout(() => {
			document.body.style.pointerEvents = 'auto';
		});
	});
</script>

<svelte:body
	oncontextmenu={(evt) => {
		// hide context menu in release builds
		if (window.location.hostname === 'tauri.localhost') {
			evt.preventDefault();
		}
	}}
/>

<Tooltip.Provider>
	<main class="bg-primary-800 relative flex flex-col overflow-hidden">
		<Menubar />
		<Contextbar />

		<div class="relative flex grow overflow-hidden">
			<nav class="border-primary-600 bg-primary-900 flex shrink-0 flex-col gap-1 border-r p-3">
				<NavbarLink to="/" icon="mdi:account-circle" label="Profile" />
				<NavbarLink to="/browse" icon="mdi:store-search" label="Browse" />
				<NavbarLink to="/config" icon="mdi:file-cog" label="Config" />
				<NavbarLink to="/modpack" icon="mdi:package-variant" label="Modpack" />
				<NavbarLink to="/prefs" icon="mdi:settings" label="Settings" />
			</nav>

			{@render children?.()}
		</div>

		<Statusbar />
		<Toasts />
	</main>

	<InstallModPopup />
	<InstallProgressPopup />
	<WelcomePopup />
</Tooltip.Provider>
