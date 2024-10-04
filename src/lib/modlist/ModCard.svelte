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
		class="h-12 w-12 rounded-md"
	/>
	<div class="flex-shrink flex-grow overflow-hidden pl-3 text-left align-middle">
		<a
			class="flex-shrink truncate font-semibold text-white hover:underline"
			href="https://thunderstore.io/c/{$activeGame?.id}/p/{author}/{name}/"
			target="_blank"
		>
			{name}
		</a>
		{#if showVersion && version !== null}
			<span class="px-1 text-slate-400">
				{version}
			</span>
		{/if}

		{#if author !== null}
			<a
				class="block truncate text-slate-400 hover:underline"
				href="https://thunderstore.io/c/{$activeGame?.id}/p/{author}/"
				target="_blank"
			>
				{author}
			</a>
		{/if}
	</div>
</div>
