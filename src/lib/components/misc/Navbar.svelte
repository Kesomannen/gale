<script lang="ts">
	import { PersistedState } from 'runed';
	import NavbarLink from './NavbarLink.svelte';

	const links = [
		{
			to: '/',
			icon: 'mdi:account-circle',
			label: 'Profile',
			tooltip: 'Manage profile'
		},
		{
			to: '/browse',
			icon: 'mdi:store-search',
			label: 'Get mods',
			tooltip: 'Install mods'
		},
		{
			to: '/config',
			icon: 'mdi:file-cog',
			label: 'Config',
			tooltip: 'Edit mod config'
		},
		{
			to: '/modpack',
			icon: 'mdi-package-variant',
			label: 'Modpack',
			tooltip: 'Export modpack'
		},
		{
			to: '/prefs',
			icon: 'mdi:cog',
			label: 'Settings',
			tooltip: 'Edit manager settings'
		}
	];

	let expanded = new PersistedState('expandNavbar', false);
</script>

<nav class="border-primary-600 bg-primary-900 relative flex shrink-0 flex-col gap-1.5 border-r p-3">
	{#each links as link (link.to)}
		<NavbarLink {...link} expanded={expanded.current} />
	{/each}

	<button
		onclick={() => (expanded.current = !expanded.current)}
		class="group absolute top-0 -right-1.5 bottom-0 w-3 cursor-col-resize"
		aria-label="resize"
	>
		<div
			class="bg-accent-500/0 group-hover:bg-accent-500 mx-auto h-full w-[3px] transition-colors delay-75 duration-150"
		></div></button
	>
</nav>
