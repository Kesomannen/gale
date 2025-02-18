<script lang="ts">
	import { activeGame } from '$lib/stores';

	export let fullName: string;
	export let showVersion = true;

	$: split = fullName.split('-');

	// kind of hacky, but we assume we either have (1) just a name, (2) name and version, or (3) all three
	$: author = split.length === 3 ? split[0] : null;
	$: name = split.length === 3 ? split[1] : split[0];
	$: version = split.length >= 2 ? (split.length === 3 ? split[2] : split[1]) : null;
</script>

<div class="flex overflow-hidden">
	<img
		src="https://gcdn.thunderstore.io/live/repository/icons/{fullName}.png"
		alt={name}
		class="size-12 rounded-sm"
	/>
	<div class="shrink overflow-hidden pl-3 text-left">
		<div class="flex gap-2">
			<a
				class="shrink truncate font-medium text-white hover:underline"
				href="https://thunderstore.io/c/{$activeGame?.slug}/p/{author}/{name}/"
				target="_blank"
				rel="noopener noreferrer"
			>
				{name.replace(/_/g, ' ')}
			</a>

			{#if showVersion && version !== null}
				<span class=" shrink-0 text-slate-400">
					{version}
				</span>
			{/if}
		</div>

		{#if author !== null}
			<a
				class="truncate text-slate-400 hover:underline"
				href="https://thunderstore.io/c/{$activeGame?.slug}/p/{author}/"
				target="_blank"
			>
				{author}
			</a>
		{/if}
	</div>
</div>
