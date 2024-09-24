<script lang="ts">
	import ModCard from './ModCard.svelte';

	export let names: string[];

	let className = '';

	export { className as class };

	// sort by name
	$: names.sort((a, b) => a.split('-')[1].localeCompare(b.split('-')[1]));
</script>

<div
	class="grid gap-x-6 gap-y-4 overflow-y-auto md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 {className}"
>
	{#each names as fullName}
		<ModCard {fullName} />
	{/each}
</div>
