<script lang="ts">
	import InputField from '$lib/components/InputField.svelte';
	import { invokeCommand } from '$lib/invoke';
	import Icon from '@iconify/svelte';

	type Package = {
		id: string;
		name: string;
		owner: string;
		description: string;
		isPinned: boolean;
		isDeprecated: boolean;
		ratingScore: number;
		downloads: number;
		hasNsfwContent: boolean;
		dateUpdated: Date;
	};

	let searchTerm = 'bepinex';
	let promise: Promise<Package[]> | null = null;

	$: if (searchTerm.length > 0) {
		promise = invokeCommand<Package[]>('plugin:gale-thunderstore|query_packages', {
			args: {
				searchTerm,
				maxResults: 10,
				communityId: 1
			}
		});
	} else {
		promise = null;
	}
</script>

<div class="p-4 overflow-y-auto w-full">
	<InputField bind:value={searchTerm} />
	{#if promise !== null}
		{#await promise}
			<h1 class="text-white">Loading...</h1>
		{:then packages}
			<div class="flex flex-col gap-2 pt-4 overflow-y-auto w-full">
				{#if packages.length === 0}
					<h1 class="text-white">No results found</h1>
				{:else}
					{#each packages as pkg}
						<button
							class="text-left p-3 rounded-xl bg-gray-700 hover:bg-gray-600 transition-colors"
						>
							<div class="flex gap-3">
								<img
									src="https://gcdn.thunderstore.io/live/repository/icons/{pkg.owner}-{pkg.name}-1.0.0.png"
									alt={pkg.name}
									class="w-14 h-14 rounded-md"
								/>
								<div>
									<span class="text-white font-bold text-xl">{pkg.name}</span>
									<span class="text-slate-300 ml-1">by {pkg.owner}</span>
									<br />
									<span class="text-slate-300">{pkg.description}</span>
								</div>
							</div>
							<div class="flex gap-4 pl-1 items-center">
								<div class="flex items-center gap-1 text-yellow-400">
									<Icon icon="mdi:star" class="inline" />
									<span>{pkg.ratingScore}</span>
								</div>
								<button
									class="inline-flex gap-2 ml-auto transition-colors bg-green-600 hover:bg-green-500 text-white font-semibold py-1.5 px-4 rounded-lg"
								>
									<Icon icon="akar-icons:download" class="w-6 h-6" />
									Install
								</button>
							</div>
						</button>
					{/each}
				{/if}
			</div>
		{:catch error}
			<p>{error.message}</p>
		{/await}
	{/if}
</div>
