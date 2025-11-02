<script lang="ts">
	import { PersistedState } from 'runed';
	import NavbarLink from './NavbarLink.svelte';
	import { m } from '$lib/paraglide/messages';

	const links = [
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
			icon: 'mdi-package-variant',
			tooltip: m.navBar_link_modpack()
		},
		{
			to: '/prefs',
			icon: 'mdi:cog',
			tooltip: m.navBar_link_prefs()
		}
	];
</script>

<nav class="border-primary-600 bg-primary-900 relative flex shrink-0 flex-col gap-1.5 border-r p-3">
	{#each links as link (link.to)}
		<NavbarLink {...link} />
	{/each}
</nav>
