<script lang="ts" generics="T extends { fullName: string }">
	import type { ClassValue } from 'clsx';
	import ModCard from './ModCard.svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		mods: T[];
		showVersion?: boolean;
		class?: ClassValue;
		cardChildren?: Snippet<[{ mod: T }]>;
	};

	let { mods, showVersion = true, class: classProp, cardChildren }: Props = $props();

	// sort by name, not author
	$effect(() => {
		mods.toSorted((a, b) => a.fullName.split('-')[1].localeCompare(b.fullName.split('-')[1]));
	});
</script>

<div class={[classProp, 'grid gap-3 overflow-y-auto']}>
	{#each mods as mod (mod.fullName)}
		<ModCard fullName={mod.fullName} {showVersion}>
			{@render cardChildren?.({ mod })}
		</ModCard>
	{/each}
</div>

<style>
	.list {
		grid-template-columns: repeat(auto-fill, minmax(17rem, 1fr));
	}
</style>
