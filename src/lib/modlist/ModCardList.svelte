<script lang="ts">
	import ModCard from './ModCard.svelte';

	export let names: string[];
	export let showVersion = true;

	let className = '';

	export { className as class };

	// sort by name, not author
	$: names.sort((a, b) => a.split('-')[1].localeCompare(b.split('-')[1]));
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
