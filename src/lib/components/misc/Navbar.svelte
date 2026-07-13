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

<nav class="border-primary-600 bg-primary-900 relative flex shrink-0 flex-col gap-2 border-r p-3">
	{#each links as link (link.to)}
		<NavbarLink {...link} />
	{/each}
</nav>
