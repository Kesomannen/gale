<script lang="ts">
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
		<span class="font-semibold text-white">
			{name}
		</span>
		<span class="px-1 text-slate-400">
			{version}
		</span>
		<div class="truncate text-slate-400">
			{author}
		</div>
	</div>
</div>
