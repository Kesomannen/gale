<script lang="ts">
	import NavbarLink from './NavbarLink.svelte';
	import { m } from '$lib/paraglide/messages';
	import games from '$lib/state/game.svelte';
	import { loaderSupportsModpacks } from '$lib/util';

	const modpacksDisabled = $derived(
		!games.active || !loaderSupportsModpacks(games.active.modLoader)
	);

	const links = $derived([
		{
			to: '/',
			icon: 'mdi:account-circle',
			tooltip: m.navBar_link_profile()
		},
		{
			to: '/browse',
			icon: 'mdi:store-search',
			tooltip: m.navBar_link_browse()
		},
		{
			to: '/config',
			icon: 'mdi:file-cog',
			tooltip: m.navBar_link_config()
		},
		{
			to: '/modpack',
			icon: 'mdi:package-variant',
			tooltip: modpacksDisabled ? m.navBar_link_modpack_disabled() : m.navBar_link_modpack(),
			outline: false,
			disabled: modpacksDisabled
		},
		{
			to: '/prefs',
			icon: 'mdi:cog',
			tooltip: m.navBar_link_prefs()
		}
	]);
</script>

<nav class="bg-primary-900 relative flex shrink-0 flex-col gap-2 px-3">
	{#each links as link (link.to)}
		<NavbarLink {...link} />
	{/each}

	<!-- An inverted rounded corner to the right of the navbar to make the main content area look like it's "cut out" of the navbar and toolbar -->
	<div class="corner-notch absolute top-0 right-0 z-10 translate-x-full"></div>
</nav>

<style>
	.corner-notch {
		--size: var(--radius-2xl);

		height: var(--size);
		width: var(--size);
		background: radial-gradient(
			circle at var(--size) var(--size),
			transparent var(--size),
			var(--color-primary-900) calc(var(--size) + 1px)
		);
	}
</style>
