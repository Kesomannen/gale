<script lang="ts">
	import Icon from '@iconify/svelte';

	let className = '';

	export { className as class };
</script>

<div class="{className} flex items-center rounded-lg bg-primary-700 py-1.5 pr-1 pl-3 text-primary-300">
	<Icon icon="mdi:lock" class="mr-2 text-xl" />
	Profile is locked
</div>
