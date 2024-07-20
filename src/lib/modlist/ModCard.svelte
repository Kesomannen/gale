<script lang="ts">
	import { activeGame } from '$lib/stores';

	export let fullName: string;

	$: split = fullName.split('-');
	$: author = split[0];
	$: name = split[1];
	$: version = split[2];
</script>

<div class="flex">
	<img
		src="https://gcdn.thunderstore.io/live/repository/icons/{fullName}.png"
		alt={name}
		class="w-12 h-12 rounded-md"
	/>
	<div class="pl-3 overflow-hidden flex-grow flex-shrink align-middle text-left">
		<a
			class="font-semibold text-white hover:underline"
			href="https://thunderstore.io/c/{$activeGame?.id}/p/{author}/{name}/"
			target="_blank"
		>
			{name}
		</a>
		<span class="px-1 text-slate-400">
			{version}
		</span>
		<a
			class="truncate text-slate-400 hover:underline block"
			href="https://thunderstore.io/c/{$activeGame?.id}/p/{author}/"
			target="_blank"
		>
			{author}
		</a>
	</div>
</div>
