<script lang="ts">
	import games from '$lib/state/game.svelte';
	import type { Snippet } from 'svelte';
	import { thunderstoreIconUrl, hexiumIconUrl } from '$lib/util';
	import { Backend } from '$lib/types';

	type Props = {
		fullName: string;
		backend: Backend;
		showVersion?: boolean;
		children?: Snippet;
	};

	let { fullName, backend, showVersion = true, children }: Props = $props();

	let split = $derived(fullName.split('-'));

	// kind of hacky, but we assume we either have (1) just a name, (2) name and version, or (3) all three
	let author = $derived(split.length === 3 ? split[0] : null);
	let name = $derived(split.length === 3 ? split[1] : split[0]);
	let version = $derived(split.length >= 2 ? (split.length === 3 ? split[2] : split[1]) : null);
	
	let authorLink = $derived(backend === Backend.Hexium ? `https://mods.valtools.org/teams/${author}` : `https://thunderstore.io/c/${games.active?.slug}/p/${author}/`);
	let modLink = $derived(backend === Backend.Hexium ? `https://mods.valtools.org/mods/1/${author}/${name}` : `https://thunderstore.io/c/${games.active?.slug}/p/${author}/${name}/`);
	let iconUrl = $derived(backend === Backend.Hexium ? hexiumIconUrl("" + author, name) : thunderstoreIconUrl(fullName))
</script>

<div class="flex overflow-hidden">
	<img
		src={iconUrl}
		alt={name}
		class="size-12 rounded-sm"
	/>
	<div class="shrink overflow-hidden pl-3 text-left">
		<div class="flex gap-2">
			<a
				class="shrink truncate font-medium text-white hover:underline"
				href="{modLink}"
				target="_blank"
				rel="noopener noreferrer"
			>
				{name.replace(/_/g, ' ')}
			</a>

			{#if showVersion && version !== null}
				<span class=" text-primary-400 shrink-0">
					{version}
				</span>
			{/if}
		</div>

		{#if author !== null}
			<a
				class="text-primary-400 truncate hover:underline"
				href="{authorLink}"
				target="_blank"
			>
				{author}
			</a>
		{/if}

		{@render children?.()}
	</div>
</div>
