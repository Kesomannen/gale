<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { ClassValue } from 'svelte/elements';

	type Props = {
		class?: ClassValue;
	};

	let { class: classProp }: Props = $props();
</script>

<Icon icon="mdi:loading" class={[classProp, 'animate-spin']} />
