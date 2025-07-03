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
	import Navbar from '$lib/menu/Navbar.svelte';

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
			<Navbar />

			{@render children?.()}
		</div>

		<Statusbar />
		<Toasts />
	</main>

	<InstallModPopup />
	<InstallProgressPopup />
	<WelcomePopup />
</Tooltip.Provider>
