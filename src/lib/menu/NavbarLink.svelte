<script lang="ts">
	import Icon from '@iconify/svelte';
	import { page } from '$app/stores';

	export let to: string;
	export let icon: string;
	export let label: string;

	$: active = $page.url.pathname === to;
</script>

<a
	href={to}
	class="relative flex items-center gap-2 rounded-md p-2 {active
		? 'text-accent-400 bg-primary-800'
		: 'text-primary-500 hover:bg-primary-800 hover:text-primary-400'}"
>
	<Icon class="text-2xl" {icon} />
	<span class="hidden lg:block {active ? 'font-semibold' : 'font-medium'}">{label}</span>
</a>
