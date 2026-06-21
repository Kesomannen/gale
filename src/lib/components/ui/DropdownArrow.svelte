<script lang="ts">
	import Icon from '@iconify/svelte';
	import clsx, { type ClassValue } from 'clsx';

	type Props = {
		open: boolean;
		class?: ClassValue;
	};

	let { open, class: classProp }: Props = $props();
</script>

<Icon
	icon="ph:caret-down-fill"
	class={clsx(
		classProp,
		open ? 'rotate-180' : 'rotate-0',
		'shrink-0 origin-center transform text-base transition-transform duration-100 ease-out'
	)}
/>
