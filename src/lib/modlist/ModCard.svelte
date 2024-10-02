<script lang="ts">
	import { activeGame } from '$lib/stores';

	export let fullName: string;
	export let showVersion = true;

	$: split = fullName.split('-');
	$: author = split[0];
	$: name = split[1];
	$: version = split[2];
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
		{#if showVersion}
			<span class="px-1 text-slate-400">
				{version}
			</span>
		{/if}

		<a
			class="block truncate text-slate-400 hover:underline"
			href="https://thunderstore.io/c/{$activeGame?.id}/p/{author}/"
			target="_blank"
		>
			{author}
		</a>
	</div>
</div>
