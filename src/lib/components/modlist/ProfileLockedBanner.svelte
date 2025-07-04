<script lang="ts">
	import Icon from '@iconify/svelte';

	type Props = {
		class?: string;
	};

	let { class: classProp }: Props = $props();
</script>

<div
	class={[
		classProp,
		'bg-primary-700 text-primary-300 flex items-center rounded-lg py-1.5 pr-1 pl-3'
	]}
>
	<Icon icon="mdi:lock" class="mr-2 text-xl" />
	Profile is locked
</div>
