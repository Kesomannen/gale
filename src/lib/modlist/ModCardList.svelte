<script lang="ts">
	import ModCard from './ModCard.svelte';

	type Props = {
		names: string[];
		showVersion?: boolean;
		class?: string;
	};

	let { names, showVersion = true, class: className = '' }: Props = $props();

	// sort by name, not author
	$effect(() => {
		names.toSorted((a, b) => a.split('-')[1].localeCompare(b.split('-')[1]));
	});
</script>

<div class="grid gap-3 overflow-y-auto {className}">
	{#each names as fullName}
		<ModCard {fullName} {showVersion} />
	{/each}
</div>

<style>
	.list {
		grid-template-columns: repeat(auto-fill, minmax(17rem, 1fr));
	}
</style>
