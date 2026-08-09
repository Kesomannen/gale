<script lang="ts">
	import Icon from '@iconify/svelte';
	import clsx from 'clsx';
	import type { ClassValue } from 'svelte/elements';

	type Props = {
		class?: ClassValue;
	};

	let { class: classProp }: Props = $props();
</script>

<Icon icon="ph:circle-notch" class={clsx(classProp, 'animate-spin')} />
