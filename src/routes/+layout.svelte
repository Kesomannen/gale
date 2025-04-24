<script lang="ts">
	import '../app.css';

	import Menubar from '$lib/menu/Menubar.svelte';
	import Contextbar from '$lib/menu/Contextbar.svelte';
	import Statusbar from '$lib/menu/Statusbar.svelte';
	import Toasts from '$lib/menu/Toasts.svelte';

	import { onMount } from 'svelte';
	import NavbarLink from '$lib/menu/NavbarLink.svelte';
	import InstallProgressPopup from '$lib/modlist/InstallProgressPopup.svelte';
	import WelcomePopup from '$lib/menu/WelcomePopup.svelte';
	import { refreshColor } from '$lib/theme';
	import InstallModPopup from '$lib/modlist/InstallModPopup.svelte';

	onMount(() => {
		refreshColor('accent');
		refreshColor('primary');
	});
</script>

<svelte:body
	on:contextmenu={(evt) => {
		// hide context menu in release builds
		if (window.location.hostname === 'tauri.localhost') {
			evt.preventDefault();
		}
	}}
/>

<main class="bg-primary-800 relative flex flex-col overflow-hidden">
	<Menubar />
	<Contextbar />

	<div class="relative flex grow overflow-hidden">
		<nav class="border-primary-600 bg-primary-900 flex shrink-0 flex-col gap-1 border-r p-2.5">
			<NavbarLink to="/" icon="mdi:account-circle" tooltip="Manage profile" />
			<NavbarLink to="/browse" icon="mdi:store-search" tooltip="Browse Thunderstore mods" />
			<NavbarLink to="/config" icon="mdi:file-cog" tooltip="Edit mod config" />
			<NavbarLink to="/modpack" icon="mdi:package-variant" tooltip="Export modpack" />
			<NavbarLink to="/sync" icon="mdi:cloud-sync" tooltip="Profile sync (beta)" />
			<NavbarLink to="/prefs" icon="mdi:settings" tooltip="Edit manager settings" />
		</nav>

		<slot />
	</div>

	<Statusbar />
	<Toasts />
</main>

<InstallModPopup />
<InstallProgressPopup />
<WelcomePopup />
