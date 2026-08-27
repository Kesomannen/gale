<script lang="ts">
	import Select from '$lib/components/ui/Select.svelte';
	import games from '$lib/state/game.svelte';

	type Props = {
		selected: string[];
		excluded: string[];
		icon: string;
		label: string;
	};

	let { selected = $bindable(), excluded = $bindable(), icon, label: text }: Props = $props();

	let items = $derived(
		games.categories
			.map((category) => ({ value: category.name, label: category.name }))
			.filter((category) => !excluded.includes(category.value))
			.toSorted()
	);
</script>

<Select {items} type="multiple" bind:value={selected} {icon} triggerClass="w-full py-1.5">
	{#snippet label()}
		{#if selected.length === 0}
			<span class="text-primary-600 dark:text-primary-300 truncate">{text}</span>
		{:else}
			<div class="mr-2 flex flex-wrap gap-1">
				{#each selected as category}
					<div
						class="text-primary-700 dark:bg-primary-800 dark:text-primary-200 bg-primary-200 overflow-hidden rounded-lg py-0.5 pr-0.5 pl-2 text-sm"
					>
						<span class="truncate overflow-hidden">{category}</span>

						<button
							class="dark:hover:bg-primary-700 hover:bg-primary-300 ml-0.5 rounded-lg px-1.5"
							onclick={(evt) => {
								evt.stopPropagation();
								selected = selected.filter((cat) => cat !== category);
							}}
						>
							x
						</button>
					</div>
				{/each}
			</div>
		{/if}
	{/snippet}
</Select>
