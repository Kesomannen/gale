<script lang="ts" generics="T extends { fullName: string, backend: Backend }">
	import type { ClassValue } from 'clsx';
	import ModCard from './ModCard.svelte';
	import type { Snippet } from 'svelte';
	import type { Backend } from '$lib/types';

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
		<ModCard fullName={mod.fullName} backend={mod.backend} {showVersion}>
			{@render cardChildren?.({ mod })}
		</ModCard>
	{/each}
</div>
