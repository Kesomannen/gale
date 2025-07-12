<script lang="ts">
	import type { ClassValue } from 'clsx';
	import ModCard from './ModCard.svelte';

	type Props = {
		names: string[];
		showVersion?: boolean;
		class?: ClassValue;
	};

	let { names, showVersion = true, class: classProp }: Props = $props();

	// sort by name, not author
	$effect(() => {
		names.toSorted((a, b) => a.split('-')[1].localeCompare(b.split('-')[1]));
	});
</script>

<div class={[classProp, 'grid gap-3 overflow-y-auto']}>
	{#each names as fullName}
		<ModCard {fullName} {showVersion} />
	{/each}
</div>

<style>
	.list {
		grid-template-columns: repeat(auto-fill, minmax(17rem, 1fr));
	}
</style>
