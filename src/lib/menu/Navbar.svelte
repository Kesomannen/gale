<script lang="ts">
	import { store } from '$lib/store.svelte';
	import NavbarLink from './NavbarLink.svelte';

	const links = [
		{
			to: '/',
			icon: 'mdi:account-circle',
			label: 'Profile'
		},
		{
			to: '/browse',
			icon: 'mdi:store-search',
			label: 'Browse'
		},
		{
			to: '/config',
			icon: 'mdi:file-cog',
			label: 'Config'
		},
		{
			to: '/modpack',
			icon: 'mdi:package-variant',
			label: 'Modpack'
		},
		{
			to: '/prefs',
			icon: 'mdi:settings',
			label: 'Settings'
		}
	];

	let expanded = $state(store.get('expandNavbar', true));

	$effect(() => store.set('expandNavbar', expanded));
</script>

<nav class="border-primary-600 bg-primary-900 relative flex shrink-0 flex-col gap-1 border-r p-3">
	{#each links as link (link.to)}
		<NavbarLink {...link} {expanded} />
	{/each}

	<button
		onclick={() => (expanded = !expanded)}
		class="group absolute top-0 -right-1.5 bottom-0 w-3 cursor-col-resize"
		aria-label="resize"
	>
		<div class="group-hover:bg-accent-500 mx-auto h-full w-[2px]"></div></button
	>
</nav>
