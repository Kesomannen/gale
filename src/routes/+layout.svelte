<script lang="ts">
	import '../app.css';

	import { Tooltip } from 'bits-ui';

	import Menubar from '$lib/components/menubar/Menubar.svelte';
	import Toolbar from '$lib/components/toolbar/Toolbar.svelte';
	import Statusbar from '$lib/components/misc/Statusbar.svelte';
	import Toasts from '$lib/components/misc/Toasts.svelte';

	import { onMount, type Snippet } from 'svelte';
	import { refreshColor, refreshFont } from '$lib/theme';
	import InstallModDialog from '$lib/components/dialogs/InstallModDialog.svelte';
	import WelcomeDialog from '$lib/components/dialogs/WelcomeDialog.svelte';
	import Navbar from '$lib/components/misc/Navbar.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { updateBanner } from '$lib/state/misc.svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { ProfileInfo, ManagedGameInfo } from '$lib/types';
	import { refreshLanguage } from '$lib/i18n';

	type Props = {
		children?: Snippet;
	};

	let { children }: Props = $props();

	let unlistenProfiles: UnlistenFn | null;
	let unlistenGames: UnlistenFn | null;

	onMount(() => {
		refreshFont();
		refreshColor('accent');
		refreshColor('primary');
		refreshLanguage();

		$effect(() => {
			profiles.active;
			updateBanner.threshold = 0;
		});

		listen<ProfileInfo>('profile_changed', (evt) => {
			profiles.updateOne(evt.payload);
		}).then((callback) => (unlistenProfiles = callback));

		listen<ManagedGameInfo>('game_changed', (evt) => {
			profiles.update(evt.payload);
		}).then((callback) => (unlistenGames = callback));

		return () => {
			unlistenProfiles?.();
			unlistenGames?.();
		};
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

<Tooltip.Provider skipDelayDuration={1} disableCloseOnTriggerClick>
	<main class="bg-primary-800 relative flex flex-col overflow-hidden">
		<Menubar />
		<Toolbar />

		<div class="relative flex grow overflow-hidden">
			<Navbar />

			{@render children?.()}
		</div>

		<Statusbar />
		<Toasts />
	</main>

	<InstallModDialog />
	<WelcomeDialog />
</Tooltip.Provider>
