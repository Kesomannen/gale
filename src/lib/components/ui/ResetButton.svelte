<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Props = HTMLButtonAttributes;

	let { class: classProp, ...restProps }: Props = $props();
</script>

<button
	class={[
		classProp,
		'text-primary-400 disabled:text-primary-500 enabled:hover:bg-primary-700 enabled:hover:text-primary-300 rounded-md p-1.5 text-lg disabled:cursor-not-allowed'
	]}
	{...restProps}
>
	<Icon icon="mdi:refresh" />
</button>
