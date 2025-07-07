<script lang="ts">
	import { PersistedState } from 'runed';
	import NavbarLink from './NavbarLink.svelte';

	const links = [
		{
			to: '/',
			icon: 'mdi:account-circle',
			tooltip: 'Manage profile'
		},
		{
			to: '/browse',
			icon: 'mdi:store-search',
			tooltip: 'Install mods'
		},
		{
			to: '/config',
			icon: 'mdi:file-cog',
			tooltip: 'Edit mod config'
		},
		{
			to: '/modpack',
			icon: 'mdi-package-variant',
			tooltip: 'Export modpack'
		},
		{
			to: '/prefs',
			icon: 'mdi:cog',
			tooltip: 'Edit manager settings'
		}
	];
</script>

<nav class="border-primary-600 bg-primary-900 relative flex shrink-0 flex-col gap-1.5 border-r p-3">
	{#each links as link (link.to)}
		<NavbarLink {...link} />
	{/each}
</nav>
